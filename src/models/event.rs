use chrono::Utc;
use chrono_tz::Asia::Shanghai;
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

impl CreateEvent {
    pub fn validate(&self) -> Result<(), String> {
        // Validate title
        if self.title.trim().is_empty() {
            return Err("Event title cannot be empty".to_string());
        }

        // Validate RFC3339 format
        let start = chrono::DateTime::parse_from_rfc3339(&self.start_time)
            .map_err(|_| format!("Invalid start_time format: {}", self.start_time))?;
        let end = chrono::DateTime::parse_from_rfc3339(&self.end_time)
            .map_err(|_| format!("Invalid end_time format: {}", self.end_time))?;

        // Validate start < end
        if start >= end {
            return Err("Event start_time must be before end_time".to_string());
        }

        // Validate recurrence_until if recurrence_rule is set
        if self.recurrence_rule.is_some() {
            if let Some(ref until) = self.recurrence_until {
                let until_dt = chrono::DateTime::parse_from_rfc3339(until)
                    .map_err(|_| format!("Invalid recurrence_until format: {}", until))?;
                if until_dt <= start {
                    return Err("recurrence_until must be after start_time".to_string());
                }
            }
        }

        Ok(())
    }
}

const VALID_EVENT_STATUSES: &[&str] = &["active", "cancelled", "expired"];

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

impl UpdateEvent {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(ref title) = self.title {
            if title.trim().is_empty() {
                return Err("Event title cannot be empty".to_string());
            }
        }

        if let Some(ref start_time) = self.start_time {
            chrono::DateTime::parse_from_rfc3339(start_time)
                .map_err(|_| format!("Invalid start_time format: {}", start_time))?;
        }
        if let Some(ref end_time) = self.end_time {
            chrono::DateTime::parse_from_rfc3339(end_time)
                .map_err(|_| format!("Invalid end_time format: {}", end_time))?;
        }

        if let Some(ref status) = self.status {
            if !VALID_EVENT_STATUSES.contains(&status.as_str()) {
                return Err(format!(
                    "Invalid status: {}. Valid values: {}",
                    status,
                    VALID_EVENT_STATUSES.join(", ")
                ));
            }
        }

        Ok(())
    }
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
    pub fn new(user_id: String, input: CreateEvent) -> Result<Self, String> {
        input.validate()?;

        let id = format!("evt_{}", Uuid::new_v4());
        let now = Utc::now().with_timezone(&Shanghai).to_rfc3339();
        let tags = input.tags.map(|t| serde_json::to_string(&t).expect("Vec<String> serialization never fails"));

        Ok(Self {
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
        })
    }
}
