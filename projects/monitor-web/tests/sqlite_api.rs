mod common;

use std::sync::Arc;

use axum::http::StatusCode;
use monitor_core::HealthChecker;
use monitor_store::SqliteRepository;
use monitor_web::app;
use serde_json::json;
use tower::ServiceExt;

/// The final checkpoint: the same router, but backed by `SQLite`.
#[tokio::test]
async fn final_router_runs_a_check_through_sqlite() {
    let url = common::serve_once("HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n").await;
    let repository = Arc::new(
        SqliteRepository::connect("sqlite::memory:")
            .await
            .expect("temporary database should open"),
    );
    let router = app(
        repository,
        HealthChecker::new(common::test_client(), common::single_attempt_policy()),
    );

    let create = router
        .clone()
        .oneshot(common::json_request(
            "POST",
            "/targets",
            &json!({ "name": "sqlite-local", "url": url }),
        ))
        .await
        .expect("router should respond");
    assert_eq!(create.status(), StatusCode::CREATED);

    let checked = router
        .clone()
        .oneshot(common::empty_request("POST", "/targets/1/checks"))
        .await
        .expect("router should respond");
    assert_eq!(checked.status(), StatusCode::OK);

    let latest = router
        .oneshot(common::empty_request("GET", "/targets/1/checks/latest"))
        .await
        .expect("router should respond");
    assert_eq!(latest.status(), StatusCode::OK);
    let json = common::response_json(latest).await;
    assert_eq!(json["name"], "sqlite-local");
    assert_eq!(json["status"], 200);
}
