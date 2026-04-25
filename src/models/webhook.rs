use chrono::Utc;
use chrono_tz::Asia::Shanghai;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const VALID_WEBHOOK_EVENTS: &[&str] = &["event.created", "event.updated", "event.deleted"];

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

impl CreateWebhook {
    pub fn validate(&self) -> Result<(), String> {
        // Validate URL format
        if self.url.trim().is_empty() {
            return Err("Webhook URL cannot be empty".to_string());
        }

        // Check if URL starts with http:// or https://
        if !self.url.starts_with("http://") && !self.url.starts_with("https://") {
            return Err("Webhook URL must start with http:// or https://".to_string());
        }

        // Validate events list is not empty
        if self.events.is_empty() {
            return Err("Webhook events list cannot be empty".to_string());
        }

        // Validate each event type
        let valid_events = VALID_WEBHOOK_EVENTS;
        for event in &self.events {
            if !valid_events.contains(&event.as_str()) {
                return Err(format!("Invalid event type: {event}. Valid types: event.created, event.updated, event.deleted"));
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateWebhook {
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

impl UpdateWebhook {
    pub fn validate(&self) -> Result<(), String> {
        // Validate URL if provided
        if let Some(ref url) = self.url {
            if !url.trim().is_empty() && !url.starts_with("http://") && !url.starts_with("https://")
            {
                return Err("Webhook URL must start with http:// or https://".to_string());
            }
        }

        // Validate events if provided
        if let Some(ref events) = self.events {
            let valid_events = VALID_WEBHOOK_EVENTS;
            for event in events {
                if !valid_events.contains(&event.as_str()) {
                    return Err(format!("Invalid event type: {event}. Valid types: event.created, event.updated, event.deleted"));
                }
            }
        }

        Ok(())
    }
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
    pub fn new(user_id: String, input: CreateWebhook) -> Result<Self, String> {
        input.validate()?;

        let id = format!("wh_{}", Uuid::new_v4());
        let events =
            serde_json::to_string(&input.events).expect("Vec<String> serialization never fails");
        let now = Utc::now().with_timezone(&Shanghai).to_rfc3339();

        Ok(Self {
            id,
            user_id,
            url: input.url,
            events,
            secret: input.secret,
            is_active: true,
            created_at: now,
        })
    }
}
