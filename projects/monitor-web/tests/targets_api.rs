use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use monitor_core::{CheckPolicy, HealthChecker};
use monitor_store::InMemoryRepository;
use monitor_web::app;
use serde_json::{Value, json};
use tower::ServiceExt;

#[tokio::test]
async fn learner_can_create_then_list_a_target_through_http() {
    let repository = Arc::new(InMemoryRepository::new());
    let checker = HealthChecker::new(reqwest::Client::new(), CheckPolicy::default());
    let router = app(repository, checker);

    let create = router
        .clone()
        .oneshot(
            Request::post("/targets")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "name": "Rust",
                        "url": "https://www.rust-lang.org"
                    })
                    .to_string(),
                ))
                .expect("request should build"),
        )
        .await
        .expect("router should respond");
    assert_eq!(create.status(), StatusCode::CREATED);

    let list = router
        .oneshot(
            Request::get("/targets")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("router should respond");
    assert_eq!(list.status(), StatusCode::OK);
    let body = to_bytes(list.into_body(), 64 * 1024)
        .await
        .expect("body should be readable");
    let json: Value = serde_json::from_slice(&body).expect("body should be JSON");
    assert_eq!(json[0]["id"], 1);
    assert_eq!(json[0]["name"], "Rust");
}
