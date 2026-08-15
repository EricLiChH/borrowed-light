use std::sync::Arc;
use std::time::Duration;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use monitor_core::{CheckPolicy, HealthChecker};
use monitor_store::InMemoryRepository;
use monitor_web::app;
use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tower::ServiceExt;

#[tokio::test]
async fn learner_can_trigger_and_retrieve_a_check_through_http() {
    let url = serve_once().await;
    let repository = Arc::new(InMemoryRepository::new());
    let policy = CheckPolicy::new(Duration::from_secs(1), 1, Duration::ZERO, 1)
        .expect("policy should be valid");
    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("test client should build");
    let router = app(repository, HealthChecker::new(client, policy));

    let create = router
        .clone()
        .oneshot(json_request(
            "POST",
            "/targets",
            &json!({ "name": "local", "url": url }),
        ))
        .await
        .expect("router should respond");
    assert_eq!(create.status(), StatusCode::CREATED);

    let check = router
        .clone()
        .oneshot(empty_request("POST", "/targets/1/checks"))
        .await
        .expect("router should respond");
    assert_eq!(check.status(), StatusCode::OK);
    let check_json = response_json(check).await;
    assert_eq!(check_json["reachable"], true);
    assert_eq!(check_json["status"], 200);

    let latest = router
        .oneshot(empty_request("GET", "/targets/1/checks/latest"))
        .await
        .expect("router should respond");
    assert_eq!(latest.status(), StatusCode::OK);
    assert_eq!(response_json(latest).await, check_json);
}

fn json_request(method: &str, uri: &str, body: &Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .expect("request should build")
}

fn empty_request(method: &str, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .expect("request should build")
}

async fn response_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("body should be readable");
    serde_json::from_slice(&body).expect("body should be JSON")
}

async fn serve_once() -> String {
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
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
            .await
            .expect("server should reply");
    });
    format!("http://{address}")
}
