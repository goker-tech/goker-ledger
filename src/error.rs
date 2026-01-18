use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("External API error: {0}")]
    ExternalApiError(String),

    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::ExternalApiError(msg) => (StatusCode::BAD_GATEWAY, msg.clone()),
            AppError::RequestError(e) => {
                tracing::error!("Request error: {:?}", e);
                (StatusCode::BAD_GATEWAY, "External request failed".to_string())
            }
            AppError::SerializationError(e) => {
                tracing::error!("Serialization error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Serialization error".to_string(),
                )
            }
            AppError::InternalError(msg) => {
                tracing::error!("Internal error: {}", msg);
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
