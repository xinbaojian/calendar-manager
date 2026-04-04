use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Event not found: {0}")]
    EventNotFound(String),

    #[error("Webhook not found: {0}")]
    WebhookNotFound(String),

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Duplicate username: {0}")]
    DuplicateUsername(String),

    #[error("Invalid recurrence rule: {0}")]
    InvalidRecurrenceRule(String),

    #[error("Webhook delivery failed: {0}")]
    WebhookDeliveryFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", self.to_string()),
            AppError::UserNotFound(_) => (StatusCode::NOT_FOUND, "USER_NOT_FOUND", self.to_string()),
            AppError::EventNotFound(_) => (StatusCode::NOT_FOUND, "EVENT_NOT_FOUND", self.to_string()),
            AppError::WebhookNotFound(_) => (StatusCode::NOT_FOUND, "WEBHOOK_NOT_FOUND", self.to_string()),
            AppError::InvalidApiKey => (StatusCode::UNAUTHORIZED, "INVALID_API_KEY", "API Key 无效".to_string()),
            AppError::InsufficientPermissions => (StatusCode::FORBIDDEN, "INSUFFICIENT_PERMISSIONS", "权限不足".to_string()),
            AppError::DuplicateUsername(_) => (StatusCode::CONFLICT, "DUPLICATE_USERNAME", self.to_string()),
            AppError::InvalidRecurrenceRule(_) => (StatusCode::BAD_REQUEST, "INVALID_RECURRENCE_RULE", self.to_string()),
            AppError::WebhookDeliveryFailed(_) => (StatusCode::INTERNAL_SERVER_ERROR, "WEBHOOK_DELIVERY_FAILED", self.to_string()),
            AppError::Serialization(_) => (StatusCode::BAD_REQUEST, "SERIALIZATION_ERROR", self.to_string()),
            AppError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", self.to_string()),
        };

        let body = json!({
            "error": {
                "code": code,
                "message": message,
                "details": {}
            }
        });

        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
