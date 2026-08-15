use std::time::Duration;

use monitor_core::{CheckPolicy, HealthChecker};
use monitor_domain::{CheckOutcome, MonitorTarget};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn learner_can_check_a_local_website_without_using_the_public_internet() {
    let url = serve_once("HTTP/1.1 204 No Content\r\nContent-Length: 0\r\n\r\n").await;
    let target = MonitorTarget::new("local", url).expect("local URL should be valid");
    let policy = CheckPolicy::new(Duration::from_secs(1), 1, Duration::ZERO, 1)
        .expect("policy should be valid");
    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("test client should build");
    let checker = HealthChecker::new(client, policy);

    let result = checker.check(&target).await;

    assert_eq!(result.outcome(), &CheckOutcome::Reachable { status: 204 });
}

async fn serve_once(response: &'static str) -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test server should bind");
    let address = listener
        .local_addr()
        .expect("listener should have an address");

    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("server should accept");
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request).await;
        stream
            .write_all(response.as_bytes())
            .await
            .expect("server should reply");
    });

    format!("http://{address}")
}
