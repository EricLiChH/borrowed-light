mod common;

use std::future::pending;
use std::time::Duration;

use monitor_core::{Backoff, CheckPolicy, HealthChecker};
use monitor_domain::{CheckFailureKind, CheckOutcome, MonitorTarget};
use tokio::net::TcpListener;
use tokio::time::Instant;

#[tokio::test(start_paused = true)]
async fn learner_can_classify_a_timeout_without_really_waiting() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test server should bind");
    let address = listener
        .local_addr()
        .expect("listener should have an address");
    tokio::spawn(async move {
        let _connection = listener.accept().await.expect("server should accept");
        pending::<()>().await;
    });

    let target = MonitorTarget::new("slow local", format!("http://{address}"))
        .expect("local URL should be valid");
    let checker = HealthChecker::new(
        common::client(),
        CheckPolicy::builder()
            .total_timeout(Duration::from_secs(2))
            .max_attempts(common::nonzero(1))
            .backoff(Backoff::none())
            .concurrency(common::nonzero(1))
            .build(),
    );

    let result = checker.check(&target).await;

    assert!(matches!(
        result.outcome(),
        CheckOutcome::Unreachable {
            kind: CheckFailureKind::Timeout,
            ..
        }
    ));
}

#[tokio::test(start_paused = true)]
async fn the_total_deadline_covers_retry_backoff() {
    // Nothing is listening, and each retry wants to wait ten seconds. The two
    // second deadline has to win, otherwise a batch of dead hosts would take
    // attempts times backoff to give up.
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test port should bind");
    let address = listener
        .local_addr()
        .expect("listener should have an address");
    drop(listener);

    let target = MonitorTarget::new("closed local port", format!("http://{address}"))
        .expect("local URL should be valid");
    let checker = HealthChecker::new(
        common::client(),
        CheckPolicy::builder()
            .total_timeout(Duration::from_secs(2))
            .max_attempts(common::nonzero(3))
            .backoff(Backoff::fixed(Duration::from_secs(10)))
            .concurrency(common::nonzero(1))
            .build(),
    );
    let started = Instant::now();

    let result = checker.check(&target).await;

    assert_eq!(started.elapsed(), Duration::from_secs(2));
    assert!(matches!(
        result.outcome(),
        CheckOutcome::Unreachable {
            kind: CheckFailureKind::Timeout,
            ..
        }
    ));
}
