use super::CalendarMCP;
use crate::mcp::models::*;
use crate::models::CreateEvent;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ErrorData;
use rmcp::{tool, tool_router};
use serde_json::Value;

/// 验证 RFC3339 时间格式
fn validate_rfc3339(time: &str, field: &str) -> Result<(), ErrorData> {
    let _: chrono::DateTime<chrono::FixedOffset> =
        chrono::DateTime::parse_from_rfc3339(time).map_err(|_| {
            ErrorData::invalid_params(format!("无效的 {} 格式: {}", field, time), None::<Value>)
        })?;
    Ok(())
}

/// 验证日程是否属于当前用户
fn verify_event_access(
    event: &crate::models::Event,
    user_id: &str,
) -> Result<(), ErrorData> {
    if event.user_id != user_id {
        return Err(ErrorData::internal_error(
            "日程不存在或无权访问".to_string(),
            None::<Value>,
        ));
    }
    Ok(())
}

/// 将 AppError 映射为 MCP ErrorData
///
/// 日程不存在返回 resource_not_found 类型，其他数据库错误返回 internal error。
fn map_event_error(e: crate::error::AppError, _context: &str) -> ErrorData {
    match &e {
        crate::error::AppError::EventNotFound(id) => ErrorData::resource_not_found(
            format!("日程不存在: {}", id),
            None::<Value>,
        ),
        _ => ErrorData::internal_error(format!("{}", e), None::<Value>),
    }
}

/// MCP 工具处理器
#[tool_router(vis = "pub")]
impl CalendarMCP {
    /// 创建日程
    #[tool(name = "create_event", description = "创建一个新的日程事件")]
    pub async fn create_event(
        &self,
        Parameters(params): Parameters<CreateEventParams>,
    ) -> Result<String, ErrorData> {
        if params.title.trim().is_empty() {
            return Err(ErrorData::invalid_params("日程标题不能为空", None::<Value>));
        }
        if params.start_time.trim().is_empty() {
            return Err(ErrorData::invalid_params("开始时间不能为空", None::<Value>));
        }
        if params.end_time.trim().is_empty() {
            return Err(ErrorData::invalid_params("结束时间不能为空", None::<Value>));
        }

        validate_rfc3339(&params.start_time, "开始时间")?;
        validate_rfc3339(&params.end_time, "结束时间")?;

        let create_event = CreateEvent {
            title: params.title,
            description: params.description,
            location: params.location,
            start_time: params.start_time,
            end_time: params.end_time,
            recurrence_rule: params.recurrence_rule,
            recurrence_until: params.recurrence_until,
            reminder_minutes: params.reminder_minutes,
            tags: params.tags,
        };

        let user_id = self.current_user().user_id.clone();
        let event_repo = self.event_repo().clone();

        let event = event_repo
            .create(user_id, create_event)
            .await
            .map_err(|e| ErrorData::internal_error(format!("创建日程失败: {}", e), None::<Value>))?;

        if let Some(ref webhook_service) = self.webhook_service() {
            let webhook_service = webhook_service.clone();
            let user_id = self.current_user().user_id.clone();
            let event_json = match serde_json::to_value(&event) {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(error = %e, "序列化日程失败，跳过 webhook 通知");
                    return Ok(event.id);
                }
            };

            tokio::spawn(async move {
                if let Err(e) = webhook_service
                    .send_event_webhook(&user_id, "event.created", event_json)
                    .await
                {
                    tracing::warn!(error = %e, "Webhook 通知发送失败");
                }
            });
        }

        Ok(event.id)
    }

    /// 查询日程列表
    #[tool(name = "list_events", description = "根据条件查询日程列表")]
    pub async fn list_events(
        &self,
        Parameters(params): Parameters<ListEventsParams>,
    ) -> Result<String, ErrorData> {
        if let Some(ref from) = params.from {
            validate_rfc3339(from, "开始日期")?;
        }
        if let Some(ref to) = params.to {
            validate_rfc3339(to, "结束日期")?;
        }
        if let Some(ref status) = params.status {
            if !matches!(status.as_str(), "active" | "expired" | "all") {
                return Err(ErrorData::invalid_params(
                    format!("无效的状态值: {}，可选值: active, expired, all", status),
                    None::<Value>,
                ));
            }
        }

        let query = crate::models::QueryEvents {
            user_id: None,
            status: params.status,
            from: params.from,
            to: params.to,
            keyword: params.keyword,
        };

        let user_id = self.current_user().user_id.clone();
        let event_repo = self.event_repo().clone();

        let events = event_repo
            .find_by_user(&user_id, query)
            .await
            .map_err(|e| ErrorData::internal_error(format!("查询日程失败: {}", e), None::<Value>))?;

        serde_json::to_string(&events)
            .map_err(|e| ErrorData::internal_error(format!("序列化日程失败: {}", e), None::<Value>))
    }

    /// 获取单个日程
    #[tool(name = "get_event", description = "根据 ID 获取单个日程详情")]
    pub async fn get_event(
        &self,
        Parameters(params): Parameters<GetEventParams>,
    ) -> Result<String, ErrorData> {
        if params.id.trim().is_empty() {
            return Err(ErrorData::invalid_params("日程 ID 不能为空", None::<Value>));
        }

        let event_repo = self.event_repo().clone();
        let user_id = self.current_user().user_id.clone();

        let event = event_repo
            .find_by_id(&params.id)
            .await
            .map_err(|e| map_event_error(e, "获取日程失败"))?;

        verify_event_access(&event, &user_id)?;

        serde_json::to_string(&event)
            .map_err(|e| ErrorData::internal_error(format!("序列化日程失败: {}", e), None::<Value>))
    }

    /// 更新日程
    #[tool(name = "update_event", description = "更新现有日程")]
    pub async fn update_event(
        &self,
        Parameters(params): Parameters<UpdateEventParams>,
    ) -> Result<String, ErrorData> {
        if params.id.trim().is_empty() {
            return Err(ErrorData::invalid_params("日程 ID 不能为空", None::<Value>));
        }

        let event_repo = self.event_repo().clone();
        let user_id = self.current_user().user_id.clone();

        let existing_event = event_repo
            .find_by_id(&params.id)
            .await
            .map_err(|e| map_event_error(e, "获取日程失败"))?;

        verify_event_access(&existing_event, &user_id)?;

        if let Some(ref t) = params.start_time {
            validate_rfc3339(t, "开始时间")?;
        }
        if let Some(ref t) = params.end_time {
            validate_rfc3339(t, "结束时间")?;
        }
        if let Some(ref t) = params.recurrence_until {
            validate_rfc3339(t, "重复结束时间")?;
        }

        let update_event = crate::models::UpdateEvent {
            title: params.title,
            description: params.description,
            location: params.location,
            start_time: params.start_time,
            end_time: params.end_time,
            status: params.status,
            recurrence_rule: params.recurrence_rule,
            recurrence_until: params.recurrence_until,
            reminder_minutes: params.reminder_minutes,
            tags: params.tags,
        };

        let updated_event = event_repo
            .update(&params.id, update_event)
            .await
            .map_err(|e| ErrorData::internal_error(format!("更新日程失败: {}", e), None::<Value>))?;

        if let Some(ref webhook_service) = self.webhook_service() {
            let webhook_service = webhook_service.clone();
            let user_id = self.current_user().user_id.clone();
            let event_json = match serde_json::to_value(&updated_event) {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(error = %e, "序列化日程失败，跳过 webhook 通知");
                    return serde_json::to_string(&updated_event)
                        .map_err(|e| ErrorData::internal_error(format!("序列化日程失败: {}", e), None::<Value>));
                }
            };

            tokio::spawn(async move {
                if let Err(e) = webhook_service
                    .send_event_webhook(&user_id, "event.updated", event_json)
                    .await
                {
                    tracing::warn!(error = %e, "Webhook 通知发送失败");
                }
            });
        }

        serde_json::to_string(&updated_event)
            .map_err(|e| ErrorData::internal_error(format!("序列化日程失败: {}", e), None::<Value>))
    }

    /// 删除日程
    #[tool(name = "delete_event", description = "删除指定日程（软删除）")]
    pub async fn delete_event(
        &self,
        Parameters(params): Parameters<DeleteEventParams>,
    ) -> Result<String, ErrorData> {
        if params.id.trim().is_empty() {
            return Err(ErrorData::invalid_params("日程 ID 不能为空", None::<Value>));
        }

        let event_repo = self.event_repo().clone();
        let user_id = self.current_user().user_id.clone();

        let existing_event = event_repo
            .find_by_id(&params.id)
            .await
            .map_err(|e| map_event_error(e, "获取日程失败"))?;

        verify_event_access(&existing_event, &user_id)?;

        event_repo.delete(&params.id).await.map_err(|e| {
            ErrorData::internal_error(format!("删除日程失败: {}", e), None::<Value>)
        })?;

        if let Some(ref webhook_service) = self.webhook_service() {
            let webhook_service = webhook_service.clone();
            let user_id = self.current_user().user_id.clone();
            let event_id = params.id;

            tokio::spawn(async move {
                if let Err(e) = webhook_service
                    .send_event_webhook(
                        &user_id,
                        "event.deleted",
                        serde_json::json!({ "id": event_id }),
                    )
                    .await
                {
                    tracing::warn!(error = %e, "Webhook 通知发送失败");
                }
            });
        }

        Ok("删除成功".to_string())
    }
}
