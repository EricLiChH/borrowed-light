use std::time::{Duration, SystemTime, UNIX_EPOCH};

use futures::{StreamExt, stream};
use monitor_domain::{CheckFailureKind, CheckResult, MonitorTarget};
use reqwest::StatusCode;
use reqwest::header::{HeaderMap, RETRY_AFTER};

use crate::CheckPolicy;

/// The reason reported when the whole check ran out of budget.
const DEADLINE_REASON: &str = "total check deadline exceeded";

/// The longest `Retry-After` value this checker will honour per attempt.
const MAX_RETRY_AFTER: Duration = Duration::from_secs(30);

/// Reuses one HTTP client and applies a checking policy.
///
/// The checker is cheap to clone — [`reqwest::Client`] is a handle to a shared
/// connection pool — and every method takes `&self`, so one instance can be
/// shared by all request handlers.
#[derive(Debug, Clone)]
pub struct HealthChecker {
    client: reqwest::Client,
    policy: CheckPolicy,
}

impl HealthChecker {
    /// Creates a checker around a reusable HTTP client.
    #[must_use]
    pub const fn new(client: reqwest::Client, policy: CheckPolicy) -> Self {
        Self { client, policy }
    }

    /// Checks one target and returns an owned result snapshot.
    ///
    /// This method is total: it returns a [`CheckResult`] that describes the
    /// failure rather than an `Err` the caller has to unpack. The single
    /// exception is cancellation — if the caller drops this future, nothing is
    /// recorded. Coming back later simply runs a fresh check.
    pub async fn check(&self, target: &MonitorTarget) -> CheckResult {
        let deadline = self.policy.total_timeout();
        match tokio::time::timeout(deadline, self.attempt_until_settled(target)).await {
            Ok(result) => result,
            Err(_elapsed) => {
                tracing::warn!(url = target.url_str(), ?deadline, "check deadline exceeded");
                CheckResult::unreachable_with(target, CheckFailureKind::Timeout, DEADLINE_REASON)
            }
        }
    }

    /// Checks a batch with bounded concurrency while preserving input order.
    ///
    /// `buffered` is the whole concurrency story: it keeps at most
    /// `policy.concurrency()` checks in flight and still yields results in the
    /// order the targets were given, so callers never re-sort afterwards.
    pub async fn check_all(&self, targets: &[MonitorTarget]) -> Vec<CheckResult> {
        stream::iter(targets.iter().cloned())
            .map(|target| {
                // Each check owns its target and its own handle to the shared
                // client and policy. Owning both is what keeps every future
                // free of the caller's borrows, so §check_all§ can be spawned
                // onto a runtime. A §HealthChecker§ clone is two words plus an
                // Arc bump, and a target clone is two small allocations.
                let checker = self.clone();
                async move { checker.check(&target).await }
            })
            .buffered(self.policy.concurrency().get())
            .collect()
            .await
    }

    /// Runs attempts until one settles the question or the policy runs out.
    async fn attempt_until_settled(&self, target: &MonitorTarget) -> CheckResult {
        let max_attempts = u32::try_from(self.policy.max_attempts().get()).unwrap_or(u32::MAX);
        let mut jitter = JitterSource::from_entropy();
        let mut last_failure: Option<(CheckFailureKind, String)> = None;

        for attempt in 1..=max_attempts {
            let is_last_attempt = attempt == max_attempts;
            tracing::debug!(url = target.url_str(), attempt, "sending check request");

            match self.client.get(target.url().clone()).send().await {
                Ok(response) => {
                    let status = response.status();
                    if is_last_attempt || !is_retryable_status(status) {
                        return CheckResult::reachable(target, status.as_u16());
                    }
                    let hint = retry_after_hint(response.headers());
                    tracing::debug!(
                        url = target.url_str(),
                        attempt,
                        status = status.as_u16(),
                        "server asked us to come back later"
                    );
                    last_failure = Some((
                        CheckFailureKind::Request,
                        format!("HTTP {} from {}", status.as_u16(), target.url_str()),
                    ));
                    self.wait_before_retry(attempt, hint, &mut jitter).await;
                }
                Err(error) => {
                    let kind = classify_transport_error(&error);
                    let reason = error.to_string();
                    if is_last_attempt || !is_retryable_transport_error(&error) {
                        return CheckResult::unreachable_with(target, kind, reason);
                    }
                    tracing::warn!(
                        url = target.url_str(),
                        attempt,
                        error = %error,
                        "retrying after a transient transport failure"
                    );
                    last_failure = Some((kind, reason));
                    self.wait_before_retry(attempt, None, &mut jitter).await;
                }
            }
        }

        let (kind, reason) = last_failure.unwrap_or_else(|| {
            (
                CheckFailureKind::Request,
                String::from("no attempt was made"),
            )
        });
        CheckResult::unreachable_with(target, kind, reason)
    }

    /// Sleeps between two attempts, preferring the server's own `Retry-After`.
    async fn wait_before_retry(
        &self,
        attempt: u32,
        retry_after: Option<Duration>,
        jitter: &mut JitterSource,
    ) {
        let backoff = self.policy.backoff();
        let computed = backoff.delay(attempt, jitter.next_fraction());
        let wait = retry_wait(computed, retry_after);

        if !wait.is_zero() {
            tracing::debug!(?wait, attempt, "waiting before the next attempt");
            tokio::time::sleep(wait).await;
        }
    }
}

/// Returns whether a status code is worth another attempt.
///
/// `408 Request Timeout` and `429 Too Many Requests` are explicit invitations to
/// retry; `5xx` means the server is broken rather than the request. Every other
/// status is an answer, and re-asking would only waste the budget.
fn is_retryable_status(status: StatusCode) -> bool {
    matches!(status.as_u16(), 408 | 429) || status.is_server_error()
}

/// Returns whether a transport error is transient.
///
/// The exclusions matter: a malformed builder, a redirect-policy violation and a
/// decoding failure are all *deterministic*, so retrying them just repeats the
/// same mistake three times and triples the latency.
fn is_retryable_transport_error(error: &reqwest::Error) -> bool {
    error.is_timeout()
        || error.is_connect()
        || error.is_body()
        || (error.is_request() && !error.is_builder() && !error.is_redirect() && !error.is_decode())
}

/// Maps a transport error onto the stable categories used by storage and the API.
///
/// This is public because the blocking CLI needs exactly the same rule. Keeping
/// one copy is what stops the two front ends from drifting apart.
#[must_use]
pub fn classify_transport_error(error: &reqwest::Error) -> CheckFailureKind {
    if error.is_timeout() {
        CheckFailureKind::Timeout
    } else if error.is_connect() {
        CheckFailureKind::Connect
    } else {
        CheckFailureKind::Request
    }
}

/// Reads the numeric form of `Retry-After`. The HTTP-date form is ignored.
///
/// Taking a `HeaderMap` instead of a whole response is what makes this
/// function unit-testable: no socket, no runtime, no clock.
fn retry_after_hint(headers: &HeaderMap) -> Option<Duration> {
    let value = headers.get(RETRY_AFTER)?.to_str().ok()?;
    let seconds = value.trim().parse::<u64>().ok()?;
    Some(Duration::from_secs(seconds))
}

/// Chooses how long to wait: the server's own hint when it sent one, else the
/// computed backoff.
///
/// Splitting this decision out of the retry loop is what lets the *rule* be
/// tested without a server, a clock or a pause. The hint is capped at
/// `MAX_RETRY_AFTER` so a confused or hostile server cannot park the checker
/// for an hour.
fn retry_wait(computed: Duration, retry_after: Option<Duration>) -> Duration {
    retry_after.map_or(computed, |hint| hint.min(MAX_RETRY_AFTER))
}

/// A tiny split-mix generator, used only to spread retries out in time.
///
/// Pulling in a random-number crate for one `f64` would be a poor trade, and
/// this keeps the checker dependency-free. The value never has to be
/// unpredictable, only *different between processes*.
#[derive(Debug)]
struct JitterSource(u64);

impl JitterSource {
    /// Seeds from the wall clock, falling back to a fixed constant.
    fn from_entropy() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0x9E37_79B9_7F4A_7C15, |elapsed| {
                u64::try_from(elapsed.as_nanos()).unwrap_or(u64::MAX)
            });
        Self(seed ^ 0x2545_F491_4F6C_DD1D)
    }

    /// Returns the next value of the split-mix sequence.
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut value = self.0;
        value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        value ^ (value >> 31)
    }

    /// Returns a fraction in `[0, 1]` with 32 bits of entropy.
    fn next_fraction(&mut self) -> f64 {
        let value = u32::try_from(self.next_u64() >> 32).unwrap_or(u32::MAX);
        f64::from(value) / f64::from(u32::MAX)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};

    use super::{MAX_RETRY_AFTER, retry_after_hint, retry_wait};

    #[test]
    fn a_numeric_retry_after_becomes_a_duration() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("2"));

        assert_eq!(retry_after_hint(&headers), Some(Duration::from_secs(2)));
    }

    #[test]
    fn a_missing_or_unparsable_retry_after_is_ignored() {
        let mut headers = HeaderMap::new();
        assert_eq!(retry_after_hint(&headers), None);

        // The HTTP-date form is legal but rare, so we fall back to backoff
        // instead of pulling in a date parser.
        headers.insert(
            RETRY_AFTER,
            HeaderValue::from_static("Wed, 21 Oct 2026 07:28:00 GMT"),
        );
        assert_eq!(retry_after_hint(&headers), None);

        headers.insert(RETRY_AFTER, HeaderValue::from_static("-3"));
        assert_eq!(retry_after_hint(&headers), None);
    }

    #[test]
    fn the_server_hint_wins_over_the_computed_backoff_but_is_capped() {
        let computed = Duration::from_millis(50);

        assert_eq!(retry_wait(computed, None), computed);
        assert_eq!(
            retry_wait(computed, Some(Duration::from_secs(2))),
            Duration::from_secs(2)
        );
        assert_eq!(
            retry_wait(computed, Some(Duration::from_secs(600))),
            MAX_RETRY_AFTER
        );
        assert_eq!(retry_wait(computed, Some(Duration::ZERO)), Duration::ZERO);
    }
}
