use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error")]
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

    #[error("Validation error: {0}")]
    ValidationError(String),

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
            AppError::Database(e) => {
                tracing::error!(error = %e, "Database operation failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", "数据库操作失败".to_string())
            }
            AppError::UserNotFound(id) => (StatusCode::NOT_FOUND, "USER_NOT_FOUND", format!("用户不存在: {id}")),
            AppError::EventNotFound(id) => (StatusCode::NOT_FOUND, "EVENT_NOT_FOUND", format!("日程不存在: {id}")),
            AppError::WebhookNotFound(id) => (StatusCode::NOT_FOUND, "WEBHOOK_NOT_FOUND", format!("Webhook 不存在: {id}")),
            AppError::InvalidApiKey => (StatusCode::UNAUTHORIZED, "INVALID_API_KEY", "API Key 无效".to_string()),
            AppError::InsufficientPermissions => (StatusCode::FORBIDDEN, "INSUFFICIENT_PERMISSIONS", "权限不足".to_string()),
            AppError::DuplicateUsername(name) => (StatusCode::CONFLICT, "DUPLICATE_USERNAME", format!("用户名已存在: {name}")),
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", msg.clone()),
            AppError::InvalidRecurrenceRule(msg) => (StatusCode::BAD_REQUEST, "INVALID_RECURRENCE_RULE", msg.clone()),
            AppError::WebhookDeliveryFailed(msg) => {
                tracing::error!(error = %msg, "Webhook delivery failed");
                (StatusCode::INTERNAL_SERVER_ERROR, "WEBHOOK_DELIVERY_FAILED", "Webhook 投递失败".to_string())
            }
            AppError::Serialization(e) => (StatusCode::BAD_REQUEST, "SERIALIZATION_ERROR", format!("数据序列化失败: {e}")),
            AppError::Io(e) => {
                tracing::error!(error = %e, "IO error");
                (StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", "服务器内部错误".to_string())
            }
        };

        let body = json!({
            "error": {
                "code": code,
                "message": message,
            }
        });

        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
