mod common;

use std::sync::Arc;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use monitor_core::HealthChecker;
use monitor_store::InMemoryRepository;
use monitor_web::app;
use serde_json::json;
use tower::ServiceExt;

fn router() -> Router {
    app(
        Arc::new(InMemoryRepository::new()),
        HealthChecker::new(common::test_client(), common::single_attempt_policy()),
    )
}

#[tokio::test]
async fn a_rejected_target_is_a_400_with_a_machine_readable_code() {
    let response = router()
        .oneshot(common::json_request(
            "POST",
            "/targets",
            &json!({ "name": "Broken", "url": "ftp://example.com" }),
        ))
        .await
        .expect("router should respond");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = common::response_json(response).await;
    assert_eq!(body["code"], "invalid_target");
    assert!(
        body["error"]
            .as_str()
            .is_some_and(|message| !message.is_empty())
    );
}

#[tokio::test]
async fn an_unknown_target_is_a_404_not_a_500() {
    // Every route that names a target must agree on what "missing" means.
    for (method, uri) in [
        ("GET", "/targets/99"),
        ("GET", "/targets/99/checks/latest"),
        ("POST", "/targets/99/checks"),
    ] {
        let response = router()
            .oneshot(common::empty_request(method, uri))
            .await
            .expect("router should respond");

        assert_eq!(response.status(), StatusCode::NOT_FOUND, "{method} {uri}");
        assert_eq!(common::response_json(response).await["code"], "not_found");
    }
}

#[tokio::test]
async fn a_created_target_can_be_fetched_by_id() {
    let router = router();
    let create = router
        .clone()
        .oneshot(common::json_request(
            "POST",
            "/targets",
            &json!({ "name": "Rust", "url": "https://www.rust-lang.org" }),
        ))
        .await
        .expect("router should respond");
    assert_eq!(create.status(), StatusCode::CREATED);

    let fetch = router
        .oneshot(common::empty_request("GET", "/targets/1"))
        .await
        .expect("router should respond");

    assert_eq!(fetch.status(), StatusCode::OK);
    assert_eq!(common::response_json(fetch).await["name"], "Rust");
}

#[tokio::test]
async fn a_malformed_body_is_rejected_before_any_handler_runs() {
    let request = Request::builder()
        .method("POST")
        .uri("/targets")
        .header("content-type", "application/json")
        .body(Body::from("{ not json"))
        .expect("request should build");

    let response = router()
        .oneshot(request)
        .await
        .expect("router should respond");

    // This rejection comes from axum's Json extractor, so the body has axum's
    // shape rather than ours. The status is still the caller's problem to fix,
    // and that is the part a client branches on.
    assert!(response.status().is_client_error());
}
