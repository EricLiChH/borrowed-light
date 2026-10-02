mod common;

use std::sync::atomic::Ordering;
use std::time::Duration;

use monitor_core::{Backoff, CheckPolicy, HealthChecker};
use monitor_domain::{CheckFailureKind, CheckOutcome, MonitorTarget};

/// A policy that makes the given number of attempts with the given wait.
fn policy(attempts: usize, backoff: Backoff) -> CheckPolicy {
    CheckPolicy::builder()
        .total_timeout(Duration::from_secs(2))
        .max_attempts(common::nonzero(attempts))
        .backoff(backoff)
        .concurrency(common::nonzero(1))
        .build()
}

#[tokio::test]
async fn learner_can_retry_a_transient_transport_failure() {
    let (url, requests) = common::serve_failure_then_success().await;
    let target = MonitorTarget::new("flaky local", url).expect("local URL should be valid");
    let checker = HealthChecker::new(common::client(), policy(2, Backoff::none()));

    let result = checker.check(&target).await;

    assert_eq!(result.outcome(), &CheckOutcome::Reachable { status: 200 });
    assert_eq!(requests.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn learner_retries_a_transient_server_error() {
    // 503 means "try again", so the second attempt is allowed to succeed.
    let (url, requests) = common::serve_status_sequence(&[503, 200]).await;
    let target = MonitorTarget::new("warming up", url).expect("local URL should be valid");
    let checker = HealthChecker::new(common::client(), policy(2, Backoff::none()));

    let result = checker.check(&target).await;

    assert_eq!(result.outcome(), &CheckOutcome::Reachable { status: 200 });
    assert_eq!(requests.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn learner_does_not_retry_a_client_error() {
    // 404 is a decision, not a hiccup: asking again would only waste the budget.
    let (url, requests) = common::serve_status_sequence(&[404]).await;
    let target = MonitorTarget::new("missing", url).expect("local URL should be valid");
    let checker = HealthChecker::new(common::client(), policy(3, Backoff::none()));

    let result = checker.check(&target).await;

    assert_eq!(result.outcome(), &CheckOutcome::Reachable { status: 404 });
    assert_eq!(requests.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn learner_still_sees_the_status_when_a_server_error_never_clears() {
    let (url, requests) = common::serve_status_sequence(&[500, 500]).await;
    let target = MonitorTarget::new("broken", url).expect("local URL should be valid");
    let checker = HealthChecker::new(common::client(), policy(2, Backoff::none()));

    let result = checker.check(&target).await;

    // A response is a response: the site answered, it just answered badly.
    assert_eq!(result.outcome(), &CheckOutcome::Reachable { status: 500 });
    assert_eq!(requests.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn learner_honours_retry_after_and_tries_again() {
    // The hint is zero here, so the retry is immediate and the test needs no
    // clock. The precedence rule itself - the hint beats the computed backoff,
    // and the hint is capped - is unit-tested in src/checker.rs, where it needs
    // neither a socket nor a timer.
    let url = common::serve_retry_after(0).await;
    let target = MonitorTarget::new("rate limited", url).expect("local URL should be valid");
    let checker = HealthChecker::new(
        common::client(),
        CheckPolicy::builder()
            .total_timeout(Duration::from_secs(10))
            .max_attempts(common::nonzero(2))
            .backoff(Backoff::fixed(Duration::from_millis(50)))
            .concurrency(common::nonzero(1))
            .build(),
    );

    let result = checker.check(&target).await;

    assert_eq!(result.outcome(), &CheckOutcome::Reachable { status: 200 });
}

#[tokio::test]
async fn learner_does_not_retry_a_redirect_policy_error() {
    let (url, requests) = common::serve_redirect_loop().await;
    let target = MonitorTarget::new("looping local", url).expect("local URL should be valid");
    let checker = HealthChecker::new(common::client(), policy(3, Backoff::none()));

    let result = checker.check(&target).await;

    assert!(matches!(
        result.outcome(),
        CheckOutcome::Unreachable {
            kind: CheckFailureKind::Request,
            ..
        }
    ));
    // One attempt is at most eleven requests: the original plus reqwest's ten redirects.
    let requests = requests.load(Ordering::SeqCst);
    assert!((2..=11).contains(&requests));
}
