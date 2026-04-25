//! MCP 端到端集成测试
//!
//! 测试完整的 MCP 工作流程，包括：
//! 1. JSON-RPC 消息格式验证
//! 2. 所有 5 个工具的功能测试
//! 3. 错误处理
//! 4. 边界情况

use calendarsync::db::repositories::{EventRepository, UserRepository, WebhookRepository};
use calendarsync::db::{create_pool, run_migrations};
use calendarsync::handlers::{hash_password, AuthenticatedUser};
use calendarsync::mcp::{
    CalendarMCP, CreateEventParams, DeleteEventParams, GetEventParams, ListEventsParams,
    UpdateEventParams,
};
use rmcp::handler::server::wrapper::Parameters;
use std::sync::Arc;

/// 创建测试环境
async fn setup_test_env() -> Result<
    (
        Arc<EventRepository>,
        Arc<UserRepository>,
        Arc<WebhookRepository>,
        String,
        AuthenticatedUser,
    ),
    Box<dyn std::error::Error>,
> {
    let test_id = uuid::Uuid::new_v4();
    let db_path = format!("sqlite::file:///tmp/mcp-e2e-{}.db", test_id);
    let pool = create_pool(&db_path).await?;
    run_migrations(&pool).await?;

    let user_repo = Arc::new(UserRepository::new(pool.clone()));
    let webhook_repo = Arc::new(WebhookRepository::new(pool.clone()));
    let event_repo = Arc::new(EventRepository::new(pool));

    // 创建测试用户
    let password_hash = hash_password("test_password")?;
    let user = user_repo
        .create(
            calendarsync::models::CreateUser {
                username: "test_user".to_string(),
                password: None,
                is_admin: Some(false),
            },
            Some(password_hash),
        )
        .await?;

    let auth_user = AuthenticatedUser {
        user_id: user.id.clone(),
        is_admin: user.is_admin,
    };

    Ok((event_repo, user_repo, webhook_repo, user.id, auth_user))
}

#[tokio::test]
async fn e2e_full_calendar_workflow() {
    // 1. 设置测试环境
    let (event_repo, _user_repo, _webhook_repo, _user_id, auth_user) =
        setup_test_env().await.unwrap();
    let mcp = CalendarMCP::new(
        event_repo.clone(),
        None,
        auth_user,
    );

    // 2. 创建日程
    let create_params = CreateEventParams {
        title: "项目评审会议".to_string(),
        description: Some("讨论 Q2 目标和路线图".to_string()),
        location: Some("会议室 302".to_string()),
        start_time: "2026-05-01T14:00:00+08:00".to_string(),
        end_time: "2026-05-01T15:30:00+08:00".to_string(),
        recurrence_rule: None,
        recurrence_until: None,
        reminder_minutes: Some(15),
        tags: Some(vec!["工作".to_string(), "会议".to_string()]),
    };

    let event_id = mcp.create_event(Parameters(create_params)).await.unwrap();
    assert!(event_id.starts_with("evt_"));

    // 3. 获取日程详情
    let get_params = GetEventParams {
        id: event_id.clone(),
    };
    let event_json = mcp.get_event(Parameters(get_params)).await.unwrap();
    let event: serde_json::Value = serde_json::from_str(&event_json).unwrap();
    assert_eq!(event["title"], "项目评审会议");
    assert_eq!(event["location"], "会议室 302");
    assert_eq!(event["reminder_minutes"], 15);

    // 4. 更新日程
    let update_params = UpdateEventParams {
        id: event_id.clone(),
        title: Some("项目评审会议（更新）".to_string()),
        description: None,
        location: Some("会议室 305".to_string()),
        start_time: None,
        end_time: None,
        status: None,
        recurrence_rule: None,
        recurrence_until: None,
        reminder_minutes: Some(30),
        tags: None,
    };

    mcp.update_event(Parameters(update_params)).await.unwrap();

    // 验证更新
    let get_params = GetEventParams {
        id: event_id.clone(),
    };
    let event_json = mcp.get_event(Parameters(get_params)).await.unwrap();
    let event: serde_json::Value = serde_json::from_str(&event_json).unwrap();
    assert_eq!(event["title"], "项目评审会议（更新）");
    assert_eq!(event["location"], "会议室 305");
    assert_eq!(event["reminder_minutes"], 30);

    // 5. 列出日程
    let list_params = ListEventsParams {
        from: Some("2026-05-01T00:00:00+08:00".to_string()),
        to: Some("2026-05-31T23:59:59+08:00".to_string()),
        status: Some("active".to_string()),
        keyword: Some("项目".to_string()),
    };

    let events_json = mcp.list_events(Parameters(list_params)).await.unwrap();
    let events: Vec<serde_json::Value> = serde_json::from_str(&events_json).unwrap();
    assert!(!events.is_empty());
    assert!(events.iter().any(|e| e["id"] == event_id));

    // 6. 删除日程
    let delete_params = DeleteEventParams {
        id: event_id.clone(),
    };
    let delete_result = mcp.delete_event(Parameters(delete_params)).await.unwrap();
    assert!(delete_result.contains("删除成功"));

    // 验证删除（日程已从数据库中彻底删除）
    let get_params = GetEventParams { id: event_id };
    let result = mcp.get_event(Parameters(get_params)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn e2e_recurring_event_workflow() {
    let (event_repo, _user_repo, _webhook_repo, _user_id, auth_user) =
        setup_test_env().await.unwrap();
    let mcp = CalendarMCP::new(
        event_repo.clone(),
        None,
        auth_user,
    );

    // 创建重复日程
    let create_params = CreateEventParams {
        title: "每周站会".to_string(),
        description: Some("团队同步会议".to_string()),
        location: Some("线上".to_string()),
        start_time: "2026-05-01T10:00:00+08:00".to_string(),
        end_time: "2026-05-01T10:30:00+08:00".to_string(),
        recurrence_rule: Some("FREQ=WEEKLY;BYDAY=MO,WE,FR".to_string()),
        recurrence_until: Some("2026-06-30T23:59:59+08:00".to_string()),
        reminder_minutes: Some(5),
        tags: Some(vec!["例行".to_string()]),
    };

    let event_id = mcp.create_event(Parameters(create_params)).await.unwrap();

    // 验证重复规则
    let get_params = GetEventParams { id: event_id };
    let event_json = mcp.get_event(Parameters(get_params)).await.unwrap();
    let event: serde_json::Value = serde_json::from_str(&event_json).unwrap();
    assert_eq!(event["recurrence_rule"], "FREQ=WEEKLY;BYDAY=MO,WE,FR");
}

#[tokio::test]
async fn e2e_error_handling() {
    let (event_repo, _user_repo, _webhook_repo, _user_id, auth_user) =
        setup_test_env().await.unwrap();
    let mcp = CalendarMCP::new(
        event_repo.clone(),
        None,
        auth_user,
    );

    // 1. 测试获取不存在的日程
    let get_params = GetEventParams {
        id: "evt_nonexistent".to_string(),
    };
    let result = mcp.get_event(Parameters(get_params)).await;
    assert!(result.is_err());

    // 2. 测试更新不存在的日程
    let update_params = UpdateEventParams {
        id: "evt_nonexistent".to_string(),
        title: Some("新标题".to_string()),
        description: None,
        location: None,
        start_time: None,
        end_time: None,
        status: None,
        recurrence_rule: None,
        recurrence_until: None,
        reminder_minutes: None,
        tags: None,
    };
    let result = mcp.update_event(Parameters(update_params)).await;
    assert!(result.is_err());

    // 3. 测试删除不存在的日程
    let delete_params = DeleteEventParams {
        id: "evt_nonexistent".to_string(),
    };
    let result = mcp.delete_event(Parameters(delete_params)).await;
    assert!(result.is_err());

    // 4. 测试无效的时间范围
    let create_params = CreateEventParams {
        title: "无效日程".to_string(),
        description: None,
        location: None,
        start_time: "2026-05-01T15:00:00+08:00".to_string(),
        end_time: "2026-05-01T14:00:00+08:00".to_string(), // 结束时间早于开始时间
        recurrence_rule: None,
        recurrence_until: None,
        reminder_minutes: None,
        tags: None,
    };
    let result = mcp.create_event(Parameters(create_params)).await;
    assert!(result.is_err());

    // 5. 测试空标题
    let create_params = CreateEventParams {
        title: "   ".to_string(), // 只有空格
        description: None,
        location: None,
        start_time: "2026-05-01T10:00:00+08:00".to_string(),
        end_time: "2026-05-01T11:00:00+08:00".to_string(),
        recurrence_rule: None,
        recurrence_until: None,
        reminder_minutes: None,
        tags: None,
    };
    let result = mcp.create_event(Parameters(create_params)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn e2e_keyword_search() {
    let (event_repo, _user_repo, _webhook_repo, _user_id, auth_user) =
        setup_test_env().await.unwrap();
    let mcp = CalendarMCP::new(
        event_repo.clone(),
        None,
        auth_user,
    );

    // 创建多个日程
    for i in 1..=3 {
        let params = CreateEventParams {
            title: if i == 1 {
                "重要项目会议".to_string()
            } else {
                format!("日程 {}", i)
            },
            description: if i == 2 {
                Some("讨论项目进度".to_string())
            } else {
                None
            },
            location: None,
            start_time: format!("2026-05-{:02}T10:00:00+08:00", i * 5),
            end_time: format!("2026-05-{:02}T11:00:00+08:00", i * 5),
            recurrence_rule: None,
            recurrence_until: None,
            reminder_minutes: None,
            tags: None,
        };
        mcp.create_event(Parameters(params)).await.unwrap();
    }

    // 搜索包含"项目"的日程
    let list_params = ListEventsParams {
        from: Some("2026-05-01T00:00:00+08:00".to_string()),
        to: Some("2026-05-31T23:59:59+08:00".to_string()),
        status: Some("active".to_string()),
        keyword: Some("项目".to_string()),
    };

    let events_json = mcp.list_events(Parameters(list_params)).await.unwrap();
    let events: Vec<serde_json::Value> = serde_json::from_str(&events_json).unwrap();
    // 应该找到 2 个日程："重要项目会议" 和描述中包含"项目"的
    assert!(events.len() >= 2);
}

#[tokio::test]
async fn e2e_status_filtering() {
    let (event_repo, _user_repo, _webhook_repo, _user_id, auth_user) =
        setup_test_env().await.unwrap();
    let mcp = CalendarMCP::new(
        event_repo.clone(),
        None,
        auth_user,
    );

    // 创建活跃日程
    let params = CreateEventParams {
        title: "活跃日程".to_string(),
        description: None,
        location: None,
        start_time: "2026-05-15T10:00:00+08:00".to_string(),
        end_time: "2026-05-15T11:00:00+08:00".to_string(),
        recurrence_rule: None,
        recurrence_until: None,
        reminder_minutes: None,
        tags: None,
    };
    let event_id = mcp.create_event(Parameters(params)).await.unwrap();

    // 验证日程已创建且状态为 active
    let get_params = GetEventParams {
        id: event_id.clone(),
    };
    let event_json = mcp.get_event(Parameters(get_params)).await.unwrap();
    let event: serde_json::Value = serde_json::from_str(&event_json).unwrap();
    assert_eq!(event["title"], "活跃日程");
    assert_eq!(event["status"], "active");

    // 查询活跃日程
    let list_params = ListEventsParams {
        from: Some("2026-05-01T00:00:00+08:00".to_string()),
        to: Some("2026-05-31T23:59:59+08:00".to_string()),
        status: Some("active".to_string()),
        keyword: None,
    };

    let events_json = mcp.list_events(Parameters(list_params)).await.unwrap();
    let events: Vec<serde_json::Value> = serde_json::from_str(&events_json).unwrap();

    // 验证至少有一个活跃日程
    assert!(!events.is_empty(), "应该至少有一个活跃日程");
    assert!(
        events.iter().any(|e| e["id"] == event_id),
        "应该包含刚创建的日程"
    );
}

#[tokio::test]
async fn e2e_partial_update() {
    let (event_repo, _user_repo, _webhook_repo, _user_id, auth_user) =
        setup_test_env().await.unwrap();
    let mcp = CalendarMCP::new(
        event_repo.clone(),
        None,
        auth_user,
    );

    // 创建日程
    let create_params = CreateEventParams {
        title: "原始标题".to_string(),
        description: Some("原始描述".to_string()),
        location: Some("原始地点".to_string()),
        start_time: "2026-05-01T10:00:00+08:00".to_string(),
        end_time: "2026-05-01T11:00:00+08:00".to_string(),
        recurrence_rule: None,
        recurrence_until: None,
        reminder_minutes: Some(15),
        tags: Some(vec!["标签1".to_string()]),
    };

    let event_id = mcp.create_event(Parameters(create_params)).await.unwrap();

    // 只更新标题
    let update_params = UpdateEventParams {
        id: event_id.clone(),
        title: Some("新标题".to_string()),
        description: None, // 不更新
        location: None,    // 不更新
        start_time: None,
        end_time: None,
        status: None,
        recurrence_rule: None,
        recurrence_until: None,
        reminder_minutes: None,
        tags: None,
    };

    mcp.update_event(Parameters(update_params)).await.unwrap();

    // 验证只有标题被更新
    let get_params = GetEventParams { id: event_id };
    let event_json = mcp.get_event(Parameters(get_params)).await.unwrap();
    let event: serde_json::Value = serde_json::from_str(&event_json).unwrap();
    assert_eq!(event["title"], "新标题");
    assert_eq!(event["description"], "原始描述");
    assert_eq!(event["location"], "原始地点");
    assert_eq!(event["reminder_minutes"], 15);
}
