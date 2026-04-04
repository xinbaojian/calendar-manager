use chrono::Utc;
use sqlx::{Pool, Sqlite};

use crate::error::{AppError, AppResult};
use crate::models::{CreateWebhook, UpdateWebhook, Webhook};

pub struct WebhookRepository {
    pool: Pool<Sqlite>,
}

impl WebhookRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, user_id: String, input: CreateWebhook) -> AppResult<Webhook> {
        let webhook = Webhook::new(user_id, input);

        sqlx::query(
            "INSERT INTO webhooks (id, user_id, url, events, secret, is_active, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .bind(&webhook.id)
        .bind(&webhook.user_id)
        .bind(&webhook.url)
        .bind(&webhook.events)
        .bind(&webhook.secret)
        .bind(webhook.is_active as i32)
        .bind(&webhook.created_at)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(webhook)
    }

    pub async fn find_by_id(&self, id: &str) -> AppResult<Webhook> {
        sqlx::query_as::<_, Webhook>("SELECT * FROM webhooks WHERE id = ?1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| AppError::WebhookNotFound(id.to_string()))
    }

    pub async fn find_by_user(&self, user_id: &str) -> AppResult<Vec<Webhook>> {
        sqlx::query_as::<_, Webhook>(
            "SELECT * FROM webhooks WHERE user_id = ?1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn find_active_by_user(&self, user_id: &str) -> AppResult<Vec<Webhook>> {
        sqlx::query_as::<_, Webhook>(
            "SELECT * FROM webhooks WHERE user_id = ?1 AND is_active = 1 ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(AppError::Database)
    }

    pub async fn update(&self, id: &str, input: UpdateWebhook) -> AppResult<Webhook> {
        let mut webhook = self.find_by_id(id).await?;

        if let Some(url) = input.url {
            webhook.url = url;
        }
        if let Some(events) = input.events {
            webhook.events = serde_json::to_string(&events).map_err(crate::error::AppError::Serialization)?;
        }
        if let Some(is_active) = input.is_active {
            webhook.is_active = is_active;
        }

        sqlx::query("UPDATE webhooks SET url = ?1, events = ?2, is_active = ?3 WHERE id = ?4")
            .bind(&webhook.url)
            .bind(&webhook.events)
            .bind(webhook.is_active as i32)
            .bind(&webhook.id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;

        Ok(webhook)
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM webhooks WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }

    pub async fn log_delivery(
        &self,
        webhook_id: &str,
        event_type: &str,
        payload: &str,
        status_code: Option<i32>,
        response_body: Option<String>,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO webhook_logs (webhook_id, event_type, payload, status_code, response_body, sent_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )
        .bind(webhook_id)
        .bind(event_type)
        .bind(payload)
        .bind(status_code)
        .bind(response_body)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }
}
