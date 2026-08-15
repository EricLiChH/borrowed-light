//! Asynchronous website checking for the course project.

use std::time::Duration;

use futures::{StreamExt, stream};
use monitor_domain::{CheckFailureKind, CheckResult, MonitorTarget};

/// Runtime limits for website checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckPolicy {
    total_timeout: Duration,
    max_attempts: usize,
    backoff: Duration,
    concurrency: usize,
}

impl CheckPolicy {
    /// Creates a checking policy.
    ///
    /// # Errors
    ///
    /// Returns [`PolicyError`] when attempts or concurrency is zero.
    pub const fn new(
        total_timeout: Duration,
        max_attempts: usize,
        backoff: Duration,
        concurrency: usize,
    ) -> Result<Self, PolicyError> {
        if max_attempts == 0 {
            return Err(PolicyError::ZeroAttempts);
        }
        if concurrency == 0 {
            return Err(PolicyError::ZeroConcurrency);
        }
        Ok(Self {
            total_timeout,
            max_attempts,
            backoff,
            concurrency,
        })
    }
}

impl Default for CheckPolicy {
    fn default() -> Self {
        Self {
            total_timeout: Duration::from_secs(5),
            max_attempts: 2,
            backoff: Duration::from_millis(100),
            concurrency: 8,
        }
    }
}

/// Why a checking policy could not be created.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyError {
    /// A check must be attempted at least once.
    ZeroAttempts,
    /// Batch checking must allow at least one in-flight request.
    ZeroConcurrency,
}

impl std::fmt::Display for PolicyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::ZeroAttempts => "max attempts must be at least one",
            Self::ZeroConcurrency => "concurrency must be at least one",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for PolicyError {}

/// Reuses one HTTP client and applies a checking policy.
#[derive(Clone)]
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
    pub async fn check(&self, target: &MonitorTarget) -> CheckResult {
        match tokio::time::timeout(self.policy.total_timeout, self.check_with_retries(target)).await
        {
            Ok(result) => result,
            Err(_) => CheckResult::unreachable_with(
                target,
                CheckFailureKind::Timeout,
                "total check deadline exceeded",
            ),
        }
    }

    async fn check_with_retries(&self, target: &MonitorTarget) -> CheckResult {
        let mut last_failure = (CheckFailureKind::Request, String::new());

        for attempt in 1..=self.policy.max_attempts {
            match self.client.get(target.url()).send().await {
                Ok(response) => {
                    return CheckResult::reachable(target, response.status().as_u16());
                }
                Err(error) => {
                    let kind = if error.is_timeout() {
                        CheckFailureKind::Timeout
                    } else if error.is_connect() {
                        CheckFailureKind::Connect
                    } else {
                        CheckFailureKind::Request
                    };
                    let retryable = is_retryable_transport_error(&error);
                    last_failure = (kind, error.to_string());
                    if !retryable {
                        break;
                    }
                }
            }

            if attempt < self.policy.max_attempts {
                tokio::time::sleep(self.policy.backoff).await;
            }
        }

        CheckResult::unreachable_with(target, last_failure.0, last_failure.1)
    }

    /// Checks a batch with bounded concurrency while preserving input order.
    pub async fn check_all(&self, targets: &[MonitorTarget]) -> Vec<CheckResult> {
        let mut indexed_results = stream::iter(targets.iter().cloned().enumerate())
            .map(|(index, target)| {
                let checker = self.clone();
                async move { (index, checker.check(&target).await) }
            })
            .buffer_unordered(self.policy.concurrency)
            .collect::<Vec<_>>()
            .await;
        indexed_results.sort_unstable_by_key(|(index, _result)| *index);
        indexed_results
            .into_iter()
            .map(|(_index, result)| result)
            .collect()
    }
}

fn is_retryable_transport_error(error: &reqwest::Error) -> bool {
    error.is_timeout()
        || error.is_connect()
        || error.is_body()
        || (error.is_request() && !error.is_builder() && !error.is_redirect() && !error.is_decode())
}
