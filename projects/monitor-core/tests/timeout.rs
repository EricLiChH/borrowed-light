use std::future::pending;
use std::time::Duration;

use monitor_core::{CheckPolicy, HealthChecker};
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
    let policy = CheckPolicy::new(Duration::from_secs(2), 1, Duration::ZERO, 1)
        .expect("policy should be valid");
    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("test client should build");
    let checker = HealthChecker::new(client, policy);

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
async fn total_timeout_includes_retry_backoff() {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test port should bind");
    let address = listener
        .local_addr()
        .expect("listener should have an address");
    drop(listener);

    let target = MonitorTarget::new("closed local port", format!("http://{address}"))
        .expect("local URL should be valid");
    let policy = CheckPolicy::new(Duration::from_secs(2), 3, Duration::from_secs(10), 1)
        .expect("policy should be valid");
    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("test client should build");
    let checker = HealthChecker::new(client, policy);
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
