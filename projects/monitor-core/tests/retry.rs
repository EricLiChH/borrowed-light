use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use monitor_core::{CheckPolicy, HealthChecker};
use monitor_domain::{CheckFailureKind, CheckOutcome, MonitorTarget};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn learner_can_retry_a_transient_transport_failure() {
    let (url, requests) = serve_failure_then_success().await;
    let target = MonitorTarget::new("flaky local", url).expect("local URL should be valid");
    let policy = CheckPolicy::new(Duration::from_secs(1), 2, Duration::ZERO, 1)
        .expect("policy should be valid");
    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("test client should build");
    let checker = HealthChecker::new(client, policy);

    let result = checker.check(&target).await;

    assert_eq!(result.outcome(), &CheckOutcome::Reachable { status: 200 });
    assert_eq!(requests.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn learner_does_not_retry_a_redirect_policy_error() {
    let (url, requests) = serve_redirect_loop().await;
    let target = MonitorTarget::new("looping local", url).expect("local URL should be valid");
    let policy = CheckPolicy::new(Duration::from_secs(2), 3, Duration::ZERO, 1)
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
            kind: CheckFailureKind::Request,
            ..
        }
    ));
    let requests = requests.load(Ordering::SeqCst);
    assert!((2..=11).contains(&requests));
}

async fn serve_failure_then_success() -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test server should bind");
    let address = listener
        .local_addr()
        .expect("listener should have an address");
    let requests = Arc::new(AtomicUsize::new(0));
    let observed_requests = Arc::clone(&requests);

    tokio::spawn(async move {
        for attempt in 1..=2 {
            let (mut stream, _) = listener.accept().await.expect("server should accept");
            observed_requests.fetch_add(1, Ordering::SeqCst);
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).await;
            if attempt == 2 {
                stream
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                    .await
                    .expect("server should reply");
            }
        }
    });

    (format!("http://{address}"), requests)
}

async fn serve_redirect_loop() -> (String, Arc<AtomicUsize>) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test server should bind");
    let address = listener
        .local_addr()
        .expect("listener should have an address");
    let url = format!("http://{address}/loop");
    let requests = Arc::new(AtomicUsize::new(0));
    let observed_requests = Arc::clone(&requests);
    let location = url.clone();

    tokio::spawn(async move {
        for _ in 0..32 {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            observed_requests.fetch_add(1, Ordering::SeqCst);
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).await;
            let response = format!(
                "HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });

    (url, requests)
}
