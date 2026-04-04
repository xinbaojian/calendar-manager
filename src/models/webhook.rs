use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Webhook {
    pub id: String,
    pub user_id: String,
    pub url: String,
    pub events: String,
    pub secret: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWebhook {
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWebhook {
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct WebhookLog {
    pub id: i64,
    pub webhook_id: String,
    pub event_type: String,
    pub payload: Option<String>,
    pub status_code: Option<i32>,
    pub response_body: Option<String>,
    pub sent_at: String,
}

impl Webhook {
    pub fn new(user_id: String, input: CreateWebhook) -> Self {
        let id = format!("wh_{}", Uuid::new_v4());
        let events = serde_json::to_string(&input.events).expect("Vec<String> serialization never fails");
        let now = Utc::now().to_rfc3339();

        Self {
            id,
            user_id,
            url: input.url,
            events,
            secret: input.secret,
            is_active: true,
            created_at: now,
        }
    }
}
