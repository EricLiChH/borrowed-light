//! HTTP interface for the course project.
//!
//! # Why the router is generic
//!
//! The router used to take `Arc<dyn MonitorRepository>`. That works, but it
//! forces the repository trait to stay object-safe, which in turn forces every
//! async method to box its future. Now that the trait returns
//! `impl Future + Send`, `dyn` is no longer available, so the state carries the
//! concrete adapter type instead.
//!
//! Callers do not notice: `with_state` erases `R`, so `app` still returns one
//! concrete [`Router`]. Tests pass an in-memory adapter, `main` passes the
//! SQLite one, and neither pays for a boxed future per query.

mod dto;
mod error;

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use monitor_core::HealthChecker;
use monitor_domain::MonitorTarget;
use monitor_store::MonitorRepository;

use crate::dto::{CheckView, CreateTarget, TargetView};
use crate::error::ApiError;

/// Builds the HTTP router around a repository and a checker.
///
/// The repository is shared through an [`Arc`] because axum clones the state for
/// every request.
pub fn app<R>(repository: Arc<R>, checker: HealthChecker) -> Router
where
    R: MonitorRepository + 'static,
{
    Router::new()
        .route("/healthz", get(healthz))
        .route("/targets", get(list_targets::<R>).post(create_target::<R>))
        .route("/targets/{id}", get(get_target::<R>))
        .route("/targets/{id}/checks", post(run_check::<R>))
        .route("/targets/{id}/checks/latest", get(latest_check::<R>))
        .with_state(AppState {
            repository,
            checker,
        })
}

struct AppState<R> {
    repository: Arc<R>,
    checker: HealthChecker,
}

// Written out on purpose. `#[derive(Clone)]` would add a `R: Clone` bound, and a
// repository never needs to be cloneable just because the state is: sharing
// already happens through the `Arc`.
impl<R> Clone for AppState<R> {
    fn clone(&self) -> Self {
        Self {
            repository: Arc::clone(&self.repository),
            checker: self.checker.clone(),
        }
    }
}

async fn healthz() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn create_target<R>(
    State(state): State<AppState<R>>,
    Json(input): Json<CreateTarget>,
) -> Result<(StatusCode, Json<TargetView>), ApiError>
where
    R: MonitorRepository + 'static,
{
    // `?` converts TargetError into a 400 with the code `invalid_target`,
    // because ApiError implements From<TargetError>.
    let target = MonitorTarget::new(input.name, input.url)?;
    let stored = state.repository.add_target(target).await?;
    Ok((StatusCode::CREATED, Json(TargetView::from(&stored))))
}

async fn list_targets<R>(
    State(state): State<AppState<R>>,
) -> Result<Json<Vec<TargetView>>, ApiError>
where
    R: MonitorRepository + 'static,
{
    let targets = state.repository.list_targets().await?;
    Ok(Json(targets.iter().map(TargetView::from).collect()))
}

async fn get_target<R>(
    State(state): State<AppState<R>>,
    Path(target_id): Path<i64>,
) -> Result<Json<TargetView>, ApiError>
where
    R: MonitorRepository + 'static,
{
    let stored = state
        .repository
        .get_target(target_id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("target {target_id} was not found")))?;
    Ok(Json(TargetView::from(&stored)))
}

async fn run_check<R>(
    State(state): State<AppState<R>>,
    Path(target_id): Path<i64>,
) -> Result<Json<CheckView>, ApiError>
where
    R: MonitorRepository + 'static,
{
    let stored = state
        .repository
        .get_target(target_id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("target {target_id} was not found")))?;

    let result = state.checker.check(stored.target()).await;
    // Record first, then answer: a client that sees a result can rely on it
    // being in the history.
    state.repository.save_result(target_id, &result).await?;
    Ok(Json(CheckView::new(target_id, &result)))
}

async fn latest_check<R>(
    State(state): State<AppState<R>>,
    Path(target_id): Path<i64>,
) -> Result<Json<CheckView>, ApiError>
where
    R: MonitorRepository + 'static,
{
    let result = state
        .repository
        .latest_result(target_id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("target {target_id} has no check result")))?;
    Ok(Json(CheckView::new(target_id, &result)))
}
