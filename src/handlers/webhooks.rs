use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};

use crate::db::repositories::WebhookRepository;
use crate::error::AppResult;
use crate::handlers::{check_user_access, AuthenticatedUser};
use crate::models::{CreateWebhook, UpdateWebhook, Webhook};

#[derive(Serialize)]
pub struct WebhookResponse {
    pub id: String,
    pub user_id: String,
    pub url: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub created_at: String,
}

impl TryFrom<Webhook> for WebhookResponse {
    type Error = serde_json::Error;

    fn try_from(webhook: Webhook) -> Result<Self, Self::Error> {
        Ok(Self {
            id: webhook.id,
            user_id: webhook.user_id,
            url: webhook.url,
            events: serde_json::from_str(&webhook.events)?,
            is_active: webhook.is_active,
            created_at: webhook.created_at,
        })
    }
}

#[derive(Deserialize)]
pub struct CreateWebhookRequest {
    pub user_id: String,
    pub webhook: CreateWebhook,
}

pub async fn create_webhook(
    State(webhook_repo): State<WebhookRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
    Json(req): Json<CreateWebhookRequest>,
) -> AppResult<Json<WebhookResponse>> {
    check_user_access(&auth, &req.user_id)?;

    let webhook = webhook_repo.create(req.user_id, req.webhook).await?;
    Ok(Json(WebhookResponse::try_from(webhook)?))
}

pub async fn list_webhooks(
    State(webhook_repo): State<WebhookRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
) -> AppResult<Json<Vec<WebhookResponse>>> {
    let webhooks = webhook_repo.find_by_user(&auth.user.id).await?;
    let response: Vec<WebhookResponse> = webhooks
        .into_iter()
        .filter_map(|w| WebhookResponse::try_from(w).ok())
        .collect();

    Ok(Json(response))
}

pub async fn get_webhook(
    State(webhook_repo): State<WebhookRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> AppResult<Json<WebhookResponse>> {
    let webhook = webhook_repo.find_by_id(&id).await?;
    check_user_access(&auth, &webhook.user_id)?;

    Ok(Json(WebhookResponse::try_from(webhook)?))
}

pub async fn update_webhook(
    State(webhook_repo): State<WebhookRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
    Json(input): Json<UpdateWebhook>,
) -> AppResult<Json<WebhookResponse>> {
    let webhook = webhook_repo.find_by_id(&id).await?;
    check_user_access(&auth, &webhook.user_id)?;

    let webhook = webhook_repo.update(&id, input).await?;
    Ok(Json(WebhookResponse::try_from(webhook)?))
}

pub async fn delete_webhook(
    State(webhook_repo): State<WebhookRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let webhook = webhook_repo.find_by_id(&id).await?;
    check_user_access(&auth, &webhook.user_id)?;

    webhook_repo.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
