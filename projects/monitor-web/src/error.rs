use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use monitor_domain::TargetError;
use monitor_store::StoreError;

/// A failed request, with a status, a stable code and a human message.
///
/// The status is decided once, at the point where the failure is understood,
/// instead of every handler guessing. The `code` is what a client should branch
/// on; `error` is for people.
pub(crate) struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    pub(crate) fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
            message: message.into(),
        }
    }

    pub(crate) fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            message: message.into(),
        }
    }

    pub(crate) fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: "target_mismatch",
            message: message.into(),
        }
    }

    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal",
            message: message.into(),
        }
    }
}

impl From<TargetError> for ApiError {
    fn from(error: TargetError) -> Self {
        Self::bad_request("invalid_target", error.to_string())
    }
}

impl From<StoreError> for ApiError {
    fn from(error: StoreError) -> Self {
        let message = error.to_string();
        match error {
            // The target the caller named does not exist: 404, not 500.
            StoreError::NotFound { .. } => Self::not_found(message),
            // The result belongs to another target: the request contradicts
            // itself, which is a 409 rather than a server fault.
            StoreError::TargetMismatch { .. } => Self::conflict(message),
            StoreError::Corrupt { .. } | StoreError::Migration { .. } | StoreError::Database(_) => {
                Self::internal(message)
            }
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        if self.status.is_server_error() {
            tracing::error!(code = self.code, message = %self.message, "request failed");
        }

        (
            self.status,
            Json(serde_json::json!({ "error": self.message, "code": self.code })),
        )
            .into_response()
    }
}
