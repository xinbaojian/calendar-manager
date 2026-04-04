use std::sync::Arc;

use crate::db::repositories::{EventRepository, UserRepository, WebhookRepository};
use crate::services::WebhookService;

#[derive(Clone)]
pub struct AppState {
    pub user_repo: Arc<UserRepository>,
    pub event_repo: Arc<EventRepository>,
    pub webhook_repo: Arc<WebhookRepository>,
    pub webhook_service: Arc<WebhookService>,
}
