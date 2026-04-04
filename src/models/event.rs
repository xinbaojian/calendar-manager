use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Event {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub recurrence_rule: Option<String>,
    pub recurrence_until: Option<String>,
    pub reminder_minutes: Option<i32>,
    pub tags: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEvent {
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub recurrence_rule: Option<String>,
    pub recurrence_until: Option<String>,
    pub reminder_minutes: Option<i32>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEvent {
    pub title: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub recurrence_rule: Option<String>,
    pub recurrence_until: Option<String>,
    pub reminder_minutes: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct QueryEvents {
    pub user_id: Option<String>,
    pub status: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub keyword: Option<String>,
}

impl Event {
    pub fn new(user_id: String, input: CreateEvent) -> Self {
        let id = format!("evt_{}", Uuid::new_v4());
        let now = Utc::now().to_rfc3339();
        let tags = input.tags.map(|t| serde_json::to_string(&t).expect("Vec<String> serialization never fails"));

        Self {
            id,
            user_id,
            title: input.title,
            description: input.description,
            location: input.location,
            start_time: input.start_time,
            end_time: input.end_time,
            recurrence_rule: input.recurrence_rule,
            recurrence_until: input.recurrence_until,
            reminder_minutes: input.reminder_minutes,
            tags,
            status: "active".to_string(),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}
