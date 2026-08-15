//! HTTP interface for the course project.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use monitor_core::HealthChecker;
use monitor_domain::{CheckFailureKind, CheckOutcome, CheckResult, MonitorTarget};
use monitor_store::{MonitorRepository, StoreError, StoredTarget};
use serde::{Deserialize, Serialize};

/// Builds the HTTP router around repository and checker interfaces.
pub fn app(repository: Arc<dyn MonitorRepository>, checker: HealthChecker) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/targets", get(list_targets).post(create_target))
        .route("/targets/{id}/checks", axum::routing::post(run_check))
        .route("/targets/{id}/checks/latest", get(latest_check))
        .with_state(AppState {
            repository,
            checker,
        })
}

#[derive(Clone)]
struct AppState {
    repository: Arc<dyn MonitorRepository>,
    checker: HealthChecker,
}

#[derive(Debug, Deserialize)]
struct CreateTarget {
    name: String,
    url: String,
}

#[derive(Debug, Serialize)]
struct TargetView {
    id: i64,
    name: String,
    url: String,
}

#[derive(Debug, Serialize)]
struct CheckView {
    target_id: i64,
    name: String,
    url: String,
    reachable: bool,
    status: Option<u16>,
    failure: Option<&'static str>,
    reason: Option<String>,
}

async fn healthz() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn create_target(
    State(state): State<AppState>,
    Json(input): Json<CreateTarget>,
) -> Result<(StatusCode, Json<TargetView>), ApiError> {
    let target = MonitorTarget::new(input.name, input.url)
        .map_err(|error| ApiError::bad_request(error.to_string()))?;
    let stored = state.repository.add_target(target).await?;
    Ok((StatusCode::CREATED, Json(target_view(&stored))))
}

async fn list_targets(State(state): State<AppState>) -> Result<Json<Vec<TargetView>>, ApiError> {
    let targets = state.repository.list_targets().await?;
    Ok(Json(targets.iter().map(target_view).collect()))
}

async fn run_check(
    State(state): State<AppState>,
    Path(target_id): Path<i64>,
) -> Result<Json<CheckView>, ApiError> {
    let stored = state
        .repository
        .get_target(target_id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("target {target_id} was not found")))?;
    let result = state.checker.check(stored.target()).await;
    state
        .repository
        .save_result(target_id, result.clone())
        .await?;
    Ok(Json(check_view(target_id, &result)))
}

async fn latest_check(
    State(state): State<AppState>,
    Path(target_id): Path<i64>,
) -> Result<Json<CheckView>, ApiError> {
    let result = state
        .repository
        .latest_result(target_id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("target {target_id} has no check result")))?;
    Ok(Json(check_view(target_id, &result)))
}

fn target_view(stored: &StoredTarget) -> TargetView {
    TargetView {
        id: stored.id(),
        name: stored.target().name().to_owned(),
        url: stored.target().url().to_owned(),
    }
}

fn check_view(target_id: i64, result: &CheckResult) -> CheckView {
    match result.outcome() {
        CheckOutcome::Reachable { status } => CheckView {
            target_id,
            name: result.target_name().to_owned(),
            url: result.target_url().to_owned(),
            reachable: true,
            status: Some(*status),
            failure: None,
            reason: None,
        },
        CheckOutcome::Unreachable { kind, reason } => CheckView {
            target_id,
            name: result.target_name().to_owned(),
            url: result.target_url().to_owned(),
            reachable: false,
            status: None,
            failure: Some(failure_kind_name(*kind)),
            reason: Some(reason.clone()),
        },
    }
}

const fn failure_kind_name(kind: CheckFailureKind) -> &'static str {
    match kind {
        CheckFailureKind::Timeout => "timeout",
        CheckFailureKind::Connect => "connect",
        CheckFailureKind::Request => "request",
    }
}

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }
}

impl From<StoreError> for ApiError {
    fn from(error: StoreError) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: error.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(serde_json::json!({ "error": self.message })),
        )
            .into_response()
    }
}
