use std::time::Duration;

use chrono::Utc;
use chrono_tz::Asia::Shanghai;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tokio::time::sleep;

use crate::error::{AppError, AppResult};
use crate::models::{Webhook, WebhookPayload};

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct WebhookService {
    webhook_repo: crate::db::repositories::WebhookRepository,
    client: reqwest::Client,
    timeout: Duration,
    max_retries: u32,
}

impl WebhookService {
    pub fn new(
        webhook_repo: crate::db::repositories::WebhookRepository,
        timeout_seconds: u64,
        max_retries: u32,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .build()
            .expect("Failed to build reqwest client");

        Self {
            webhook_repo,
            client,
            timeout: Duration::from_secs(timeout_seconds),
            max_retries,
        }
    }

    pub async fn send_event_webhook(
        &self,
        user_id: &str,
        event_type: &str,
        data: serde_json::Value,
    ) -> AppResult<()> {
        let webhooks = self.webhook_repo.find_active_by_user(user_id).await?;

        for webhook in webhooks {
            let events: Vec<String> = serde_json::from_str(&webhook.events).unwrap();
            if !events.contains(&event_type.to_string()) {
                continue;
            }

            let payload = WebhookPayload {
                event_type: event_type.to_string(),
                data: data.clone(),
                timestamp: Utc::now().with_timezone(&Shanghai).to_rfc3339(),
            };

            let payload_json = serde_json::to_string(&payload)?;

            if let Err(e) = self.send_with_retry(&webhook, &payload_json).await {
                tracing::error!("Webhook {} failed: {}", webhook.id, e);
            }
        }

        Ok(())
    }

    async fn send_with_retry(&self, webhook: &Webhook, payload: &str) -> AppResult<()> {
        let mut last_error = None;

        for attempt in 1..=self.max_retries {
            let mut request = self
                .client
                .post(&webhook.url)
                .header("Content-Type", "application/json")
                .body(payload.to_string())
                .timeout(self.timeout);

            if let Some(secret) = &webhook.secret {
                let signature = self.sign(payload, secret)?;
                request = request.header("X-Webhook-Signature", format!("sha256={}", signature));
            }

            match request.send().await {
                Ok(response) => {
                    let status = response.status().as_u16();
                    let body = response.text().await.unwrap_or_default();

                    self.webhook_repo
                        .log_delivery(
                            &webhook.id,
                            "event",
                            payload,
                            Some(status as i32),
                            Some(body.clone()),
                        )
                        .await?;

                    if (200..300).contains(&status) {
                        return Ok(());
                    }

                    last_error = Some(AppError::WebhookDeliveryFailed(format!("Status: {status}")));
                }
                Err(e) => {
                    last_error = Some(AppError::WebhookDeliveryFailed(e.to_string()));
                }
            }

            if attempt < self.max_retries {
                sleep(Duration::from_secs(2u64.pow(attempt - 1))).await;
            }
        }

        Err(last_error.unwrap())
    }

    fn sign(&self, payload: &str, secret: &str) -> AppResult<String> {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|_| AppError::WebhookDeliveryFailed("Invalid secret".to_string()))?;
        mac.update(payload.as_bytes());
        Ok(hex::encode(mac.finalize().into_bytes()))
    }
}
