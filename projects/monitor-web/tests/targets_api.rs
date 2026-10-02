mod common;

use std::sync::Arc;

use axum::http::StatusCode;
use monitor_core::HealthChecker;
use monitor_store::InMemoryRepository;
use monitor_web::app;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn learner_can_create_then_list_a_target_through_http() {
    let router = app(
        Arc::new(InMemoryRepository::new()),
        HealthChecker::new(common::test_client(), common::single_attempt_policy()),
    );

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

    let list = router
        .oneshot(common::empty_request("GET", "/targets"))
        .await
        .expect("router should respond");
    assert_eq!(list.status(), StatusCode::OK);
    let json = common::response_json(list).await;
    assert_eq!(json[0]["id"], 1);
    assert_eq!(json[0]["name"], "Rust");
    // The stored URL is the normalized one, path separator included.
    assert_eq!(json[0]["url"], "https://www.rust-lang.org/");
}
