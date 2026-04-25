use calendarsync::db::repositories::{EventRepository, UserRepository};
use calendarsync::db::{create_pool, run_migrations};
use calendarsync::handlers::{hash_password, AuthenticatedUser};
use calendarsync::mcp::{CalendarMCP, CreateEventParams};
use rmcp::handler::server::wrapper::Parameters;
use std::sync::Arc;

#[tokio::test]
async fn test_mcp_create_event() {
    // 使用内存数据库
    let test_id = uuid::Uuid::new_v4();
    let db_path = format!("sqlite::file:///tmp/mcp-test-{}.db", test_id);
    let pool = create_pool(&db_path).await.unwrap();
    run_migrations(&pool).await.unwrap();

    let user_repo = Arc::new(UserRepository::new(pool.clone()));

    let _webhook_repo = Arc::new(calendarsync::db::repositories::WebhookRepository::new(
        pool.clone(),
    ));
    let event_repo = Arc::new(EventRepository::new(pool));

    // 创建测试用户
    let password_hash = hash_password("test_password").unwrap();
    let user = user_repo
        .create(
            calendarsync::models::CreateUser {
                username: "test_user".to_string(),
                password: None,
                is_admin: Some(false),
            },
            Some(password_hash),
        )
        .await
        .unwrap();

    let auth_user = AuthenticatedUser {
        user_id: user.id.clone(),
        is_admin: user.is_admin,
    };

    let mcp = CalendarMCP::new(event_repo, None, auth_user);

    // 测试创建日程
    let params = CreateEventParams {
        title: "测试会议".to_string(),
        description: Some("MCP 测试".to_string()),
        location: Some("线上".to_string()),
        start_time: "2026-04-25T10:00:00+08:00".to_string(),
        end_time: "2026-04-25T11:00:00+08:00".to_string(),
        recurrence_rule: None,
        recurrence_until: None,
        reminder_minutes: Some(15),
        tags: Some(vec!["test".to_string()]),
    };

    let result = mcp.create_event(Parameters(params)).await;
    assert!(result.is_ok());

    let event_id = result.unwrap();
    assert!(!event_id.is_empty());
    assert!(event_id.starts_with("evt_"));
}

#[tokio::test]
async fn test_mcp_create_event_with_invalid_time() {
    // 使用内存数据库
    let test_id = uuid::Uuid::new_v4();
    let db_path = format!("sqlite::file:///tmp/mcp-test-{}.db", test_id);
    let pool = create_pool(&db_path).await.unwrap();
    run_migrations(&pool).await.unwrap();

    let user_repo = Arc::new(UserRepository::new(pool.clone()));

    let _webhook_repo = Arc::new(calendarsync::db::repositories::WebhookRepository::new(
        pool.clone(),
    ));
    let event_repo = Arc::new(EventRepository::new(pool));

    // 创建测试用户
    let password_hash = hash_password("test_password").unwrap();
    let user = user_repo
        .create(
            calendarsync::models::CreateUser {
                username: "test_user".to_string(),
                password: None,
                is_admin: Some(false),
            },
            Some(password_hash),
        )
        .await
        .unwrap();

    let auth_user = AuthenticatedUser {
        user_id: user.id.clone(),
        is_admin: user.is_admin,
    };

    let mcp = CalendarMCP::new(event_repo, None, auth_user);

    // 测试创建时间错误的日程
    let params = CreateEventParams {
        title: "测试会议".to_string(),
        description: Some("MCP 测试".to_string()),
        location: Some("线上".to_string()),
        start_time: "2026-04-25T11:00:00+08:00".to_string(),
        end_time: "2026-04-25T10:00:00+08:00".to_string(), // 结束时间早于开始时间
        recurrence_rule: None,
        recurrence_until: None,
        reminder_minutes: Some(15),
        tags: Some(vec!["test".to_string()]),
    };

    let result = mcp.create_event(Parameters(params)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mcp_create_event_with_recurrence() {
    // 使用内存数据库
    let test_id = uuid::Uuid::new_v4();
    let db_path = format!("sqlite::file:///tmp/mcp-test-{}.db", test_id);
    let pool = create_pool(&db_path).await.unwrap();
    run_migrations(&pool).await.unwrap();

    let user_repo = Arc::new(UserRepository::new(pool.clone()));

    let _webhook_repo = Arc::new(calendarsync::db::repositories::WebhookRepository::new(
        pool.clone(),
    ));
    let event_repo = Arc::new(EventRepository::new(pool));

    // 创建测试用户
    let password_hash = hash_password("test_password").unwrap();
    let user = user_repo
        .create(
            calendarsync::models::CreateUser {
                username: "test_user".to_string(),
                password: None,
                is_admin: Some(false),
            },
            Some(password_hash),
        )
        .await
        .unwrap();

    let auth_user = AuthenticatedUser {
        user_id: user.id.clone(),
        is_admin: user.is_admin,
    };

    let mcp = CalendarMCP::new(event_repo, None, auth_user);

    // 测试创建重复日程
    let params = CreateEventParams {
        title: "每日站会".to_string(),
        description: Some("每天早上10点".to_string()),
        location: Some("会议室A".to_string()),
        start_time: "2026-04-25T10:00:00+08:00".to_string(),
        end_time: "2026-04-25T10:30:00+08:00".to_string(),
        recurrence_rule: Some("FREQ=DAILY".to_string()),
        recurrence_until: Some("2026-05-01T23:59:59+08:00".to_string()),
        reminder_minutes: Some(5),
        tags: Some(vec!["会议".to_string(), "日常".to_string()]),
    };

    let result = mcp.create_event(Parameters(params)).await;
    assert!(result.is_ok());

    let event_id = result.unwrap();
    assert!(!event_id.is_empty());
}
