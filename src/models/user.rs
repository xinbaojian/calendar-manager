use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub api_key: String,
    pub is_admin: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub is_admin: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub username: Option<String>,
}

impl User {
    pub fn new(username: String, is_admin: bool) -> Self {
        let id = format!("usr_{}", Uuid::new_v4());
        let api_key = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        Self {
            id,
            username,
            api_key,
            is_admin,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}
