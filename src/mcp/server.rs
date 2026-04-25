use rmcp::{tool_handler, ServerHandler};
use std::sync::Arc;

use crate::db::repositories::{EventRepository, UserRepository, WebhookRepository};
use crate::handlers::AuthenticatedUser;
use crate::services::WebhookService;

/// Calendar MCP 服务器
///
/// 实现 MCP 协议的服务器端，提供日程管理工具。
#[derive(Clone)]
pub struct CalendarMCP {
    event_repo: Arc<EventRepository>,
    user_repo: Arc<UserRepository>,
    webhook_repo: Arc<WebhookRepository>,
    webhook_service: Option<WebhookService>,
    current_user: AuthenticatedUser,
}

#[tool_handler(
    router = Self::tool_router(),
    name = "calendar-mcp",
    version = "0.1.0",
    instructions = "CalendarSync MCP Server - 日程管理服务"
)]
impl ServerHandler for CalendarMCP {}

impl CalendarMCP {
    pub fn new(
        event_repo: Arc<EventRepository>,
        user_repo: Arc<UserRepository>,
        webhook_repo: Arc<WebhookRepository>,
        webhook_service: Option<WebhookService>,
        current_user: AuthenticatedUser,
    ) -> Self {
        Self {
            event_repo,
            user_repo,
            webhook_repo,
            webhook_service,
            current_user,
        }
    }

    pub fn event_repo(&self) -> &Arc<EventRepository> {
        &self.event_repo
    }

    pub fn user_repo(&self) -> &Arc<UserRepository> {
        &self.user_repo
    }

    pub fn webhook_repo(&self) -> &Arc<WebhookRepository> {
        &self.webhook_repo
    }

    pub fn webhook_service(&self) -> &Option<WebhookService> {
        &self.webhook_service
    }

    pub fn current_user(&self) -> &AuthenticatedUser {
        &self.current_user
    }
}
