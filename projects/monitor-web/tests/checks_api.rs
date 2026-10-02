mod common;

use std::sync::Arc;

use axum::http::StatusCode;
use monitor_core::HealthChecker;
use monitor_store::InMemoryRepository;
use monitor_web::app;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn learner_can_trigger_and_retrieve_a_check_through_http() {
    let url = common::serve_once("HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").await;
    let router = app(
        Arc::new(InMemoryRepository::new()),
        HealthChecker::new(common::test_client(), common::single_attempt_policy()),
    );

    let create = router
        .clone()
        .oneshot(common::json_request(
            "POST",
            "/targets",
            &json!({ "name": "local", "url": url }),
        ))
        .await
        .expect("router should respond");
    assert_eq!(create.status(), StatusCode::CREATED);

    let check = router
        .clone()
        .oneshot(common::empty_request("POST", "/targets/1/checks"))
        .await
        .expect("router should respond");
    assert_eq!(check.status(), StatusCode::OK);
    let check_json = common::response_json(check).await;
    assert_eq!(check_json["reachable"], true);
    assert_eq!(check_json["status"], 200);

    // The response to a check and the stored history have to agree, otherwise a
    // client that re-reads the latest result would see something different.
    let latest = router
        .oneshot(common::empty_request("GET", "/targets/1/checks/latest"))
        .await
        .expect("router should respond");
    assert_eq!(latest.status(), StatusCode::OK);
    assert_eq!(common::response_json(latest).await, check_json);
}
