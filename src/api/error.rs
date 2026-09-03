use crate::services::ServiceError;
use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorEnvelope {
    error: ErrorBody,
    request_id: String,
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

#[derive(Debug)]
pub(super) struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
    request_id: String,
}

impl ApiError {
    pub(super) fn new(
        status: StatusCode,
        code: &'static str,
        message: impl Into<String>,
        request_id: impl Into<String>,
    ) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            request_id: request_id.into(),
        }
    }
    pub(super) fn from_service(error: ServiceError, request_id: &str) -> Self {
        let (status, code, message) = match error {
            ServiceError::InvalidInput(message) => {
                (StatusCode::BAD_REQUEST, "invalid_request", message)
            }
            ServiceError::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "The requested resource was not found.".into(),
            ),
            ServiceError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "authentication_required",
                "Sign in with GitHub to continue.".into(),
            ),
            ServiceError::Forbidden => (
                StatusCode::FORBIDDEN,
                "run_not_owned",
                "This run belongs to another user or is anonymous.".into(),
            ),
            ServiceError::Conflict => (
                StatusCode::CONFLICT,
                "run_not_completed",
                "The run must complete before it can be submitted.".into(),
            ),
            ServiceError::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "database_unavailable",
                "The service is temporarily unavailable.".into(),
            ),
            ServiceError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "The request could not be completed.".into(),
            ),
        };
        Self::new(status, code, message, request_id)
    }
    pub(super) fn invalid_json(error: JsonRejection, request_id: &str) -> Self {
        tracing::debug!(%error, "invalid JSON request body");
        Self::new(
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "Request body must be valid JSON with all required fields.",
            request_id,
        )
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorEnvelope {
                error: ErrorBody {
                    code: self.code,
                    message: self.message,
                },
                request_id: self.request_id,
            }),
        )
            .into_response()
    }
}
