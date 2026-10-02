//! Fixtures shared by the monitor-web integration tests.
//!
//! Every integration test file compiles its own copy of this module, so a
//! helper used by only one file would otherwise be reported as dead code.
#![allow(dead_code)]

use std::num::NonZeroUsize;
use std::time::Duration;

use axum::body::{Body, to_bytes};
use axum::http::Request;
use monitor_core::{Backoff, CheckPolicy};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

/// A policy that makes exactly one attempt and never waits.
pub fn single_attempt_policy() -> CheckPolicy {
    CheckPolicy::builder()
        .total_timeout(Duration::from_secs(1))
        .max_attempts(NonZeroUsize::MIN)
        .backoff(Backoff::none())
        .concurrency(NonZeroUsize::MIN)
        .build()
}

/// An HTTP client that never consults a proxy.
pub fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("test client should build")
}

/// Builds a JSON request with the matching content type.
pub fn json_request(method: &str, uri: &str, body: &Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()))
        .expect("request should build")
}

/// Builds a request without a body.
pub fn empty_request(method: &str, uri: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())
        .expect("request should build")
}

/// Reads a response body as JSON.
pub async fn response_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("body should be readable");
    serde_json::from_slice(&body).expect("body should be JSON")
}

/// Serves one HTTP response from a local port, then stops.
pub async fn serve_once(response: &'static str) -> String {
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
        let _ = stream.write_all(response.as_bytes()).await;
    });
    format!("http://{address}")
}
