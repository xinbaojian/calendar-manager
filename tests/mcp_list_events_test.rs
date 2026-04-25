use calendarsync::db::repositories::{EventRepository, UserRepository};
use calendarsync::db::{create_pool, run_migrations};
use calendarsync::handlers::{hash_password, AuthenticatedUser};
use calendarsync::mcp::{CalendarMCP, CreateEventParams, ListEventsParams};
use rmcp::handler::server::wrapper::Parameters;
use std::sync::Arc;

#[tokio::test]
async fn test_mcp_list_events() {
    // 使用内存数据库
    let test_id = uuid::Uuid::new_v4();
    let db_path = format!("sqlite::file:///tmp/mcp-test-{}.db", test_id);
    let pool = create_pool(&db_path).await.unwrap();
    run_migrations(&pool).await.unwrap();

    let user_repo = Arc::new(UserRepository::new(pool.clone()));

    let webhook_repo = Arc::new(calendarsync::db::repositories::WebhookRepository::new(
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

    let mcp = CalendarMCP::new(event_repo, user_repo, webhook_repo.clone(), None, auth_user);

    // 先创建几个测试日程
    let _ = mcp
        .create_event(Parameters(CreateEventParams {
            title: "会议A".to_string(),
            description: Some("重要会议".to_string()),
            location: Some("会议室1".to_string()),
            start_time: "2026-04-25T10:00:00+08:00".to_string(),
            end_time: "2026-04-25T11:00:00+08:00".to_string(),
            recurrence_rule: None,
            recurrence_until: None,
            reminder_minutes: Some(15),
            tags: Some(vec!["work".to_string()]),
        }))
        .await;

    let _ = mcp
        .create_event(Parameters(CreateEventParams {
            title: "会议B".to_string(),
            description: Some("普通会议".to_string()),
            location: Some("会议室2".to_string()),
            start_time: "2026-04-26T14:00:00+08:00".to_string(),
            end_time: "2026-04-26T15:00:00+08:00".to_string(),
            recurrence_rule: None,
            recurrence_until: None,
            reminder_minutes: None,
            tags: Some(vec!["work".to_string()]),
        }))
        .await;

    // 测试查询所有日程
    let params = ListEventsParams {
        from: None,
        to: None,
        status: None,
        keyword: None,
    };

    let result = mcp.list_events(Parameters(params)).await;
    assert!(result.is_ok());

    let events_json = result.unwrap();
    let events: Vec<serde_json::Value> = serde_json::from_str(&events_json).unwrap();
    assert_eq!(events.len(), 2);
}

#[tokio::test]
async fn test_mcp_list_events_with_date_filter() {
    // 使用内存数据库
    let test_id = uuid::Uuid::new_v4();
    let db_path = format!("sqlite::file:///tmp/mcp-test-{}.db", test_id);
    let pool = create_pool(&db_path).await.unwrap();
    run_migrations(&pool).await.unwrap();

    let user_repo = Arc::new(UserRepository::new(pool.clone()));

    let webhook_repo = Arc::new(calendarsync::db::repositories::WebhookRepository::new(
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

    let mcp = CalendarMCP::new(event_repo, user_repo, webhook_repo.clone(), None, auth_user);

    // 创建不同日期的日程
    let _ = mcp
        .create_event(Parameters(CreateEventParams {
            title: "4月25日会议".to_string(),
            description: None,
            location: None,
            start_time: "2026-04-25T10:00:00+08:00".to_string(),
            end_time: "2026-04-25T11:00:00+08:00".to_string(),
            recurrence_rule: None,
            recurrence_until: None,
            reminder_minutes: None,
            tags: None,
        }))
        .await;

    let _ = mcp
        .create_event(Parameters(CreateEventParams {
            title: "4月26日会议".to_string(),
            description: None,
            location: None,
            start_time: "2026-04-26T10:00:00+08:00".to_string(),
            end_time: "2026-04-26T11:00:00+08:00".to_string(),
            recurrence_rule: None,
            recurrence_until: None,
            reminder_minutes: None,
            tags: None,
        }))
        .await;

    let _ = mcp
        .create_event(Parameters(CreateEventParams {
            title: "4月27日会议".to_string(),
            description: None,
            location: None,
            start_time: "2026-04-27T10:00:00+08:00".to_string(),
            end_time: "2026-04-27T11:00:00+08:00".to_string(),
            recurrence_rule: None,
            recurrence_until: None,
            reminder_minutes: None,
            tags: None,
        }))
        .await;

    // 测试日期范围查询
    let params = ListEventsParams {
        from: Some("2026-04-25T00:00:00+08:00".to_string()),
        to: Some("2026-04-26T23:59:59+08:00".to_string()),
        status: None,
        keyword: None,
    };

    let result = mcp.list_events(Parameters(params)).await;
    assert!(result.is_ok());

    let events_json = result.unwrap();
    let events: Vec<serde_json::Value> = serde_json::from_str(&events_json).unwrap();
    assert_eq!(events.len(), 2);
}

#[tokio::test]
async fn test_mcp_list_events_with_keyword() {
    // 使用内存数据库
    let test_id = uuid::Uuid::new_v4();
    let db_path = format!("sqlite::file:///tmp/mcp-test-{}.db", test_id);
    let pool = create_pool(&db_path).await.unwrap();
    run_migrations(&pool).await.unwrap();

    let user_repo = Arc::new(UserRepository::new(pool.clone()));

    let webhook_repo = Arc::new(calendarsync::db::repositories::WebhookRepository::new(
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

    let mcp = CalendarMCP::new(event_repo, user_repo, webhook_repo.clone(), None, auth_user);

    // 创建包含不同关键词的日程
    let _ = mcp
        .create_event(Parameters(CreateEventParams {
            title: "重要会议".to_string(),
            description: Some("讨论项目进度".to_string()),
            location: None,
            start_time: "2026-04-25T10:00:00+08:00".to_string(),
            end_time: "2026-04-25T11:00:00+08:00".to_string(),
            recurrence_rule: None,
            recurrence_until: None,
            reminder_minutes: None,
            tags: None,
        }))
        .await;

    let _ = mcp
        .create_event(Parameters(CreateEventParams {
            title: "团队建设".to_string(),
            description: Some("团建活动".to_string()),
            location: None,
            start_time: "2026-04-26T10:00:00+08:00".to_string(),
            end_time: "2026-04-26T11:00:00+08:00".to_string(),
            recurrence_rule: None,
            recurrence_until: None,
            reminder_minutes: None,
            tags: None,
        }))
        .await;

    // 测试关键词搜索
    let params = ListEventsParams {
        from: None,
        to: None,
        status: None,
        keyword: Some("会议".to_string()),
    };

    let result = mcp.list_events(Parameters(params)).await;
    assert!(result.is_ok());

    let events_json = result.unwrap();
    let events: Vec<serde_json::Value> = serde_json::from_str(&events_json).unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["title"], "重要会议");
}

#[tokio::test]
async fn test_mcp_list_events_with_invalid_format() {
    // 使用内存数据库
    let test_id = uuid::Uuid::new_v4();
    let db_path = format!("sqlite::file:///tmp/mcp-test-{}.db", test_id);
    let pool = create_pool(&db_path).await.unwrap();
    run_migrations(&pool).await.unwrap();

    let user_repo = Arc::new(UserRepository::new(pool.clone()));

    let webhook_repo = Arc::new(calendarsync::db::repositories::WebhookRepository::new(
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

    let mcp = CalendarMCP::new(event_repo, user_repo, webhook_repo.clone(), None, auth_user);

    // 测试无效的时间格式
    let params = ListEventsParams {
        from: Some("invalid-date".to_string()),
        to: None,
        status: None,
        keyword: None,
    };

    let result = mcp.list_events(Parameters(params)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mcp_list_events_with_invalid_status() {
    // 使用内存数据库
    let test_id = uuid::Uuid::new_v4();
    let db_path = format!("sqlite::file:///tmp/mcp-test-{}.db", test_id);
    let pool = create_pool(&db_path).await.unwrap();
    run_migrations(&pool).await.unwrap();

    let user_repo = Arc::new(UserRepository::new(pool.clone()));

    let webhook_repo = Arc::new(calendarsync::db::repositories::WebhookRepository::new(
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

    let mcp = CalendarMCP::new(event_repo, user_repo, webhook_repo.clone(), None, auth_user);

    // 测试无效的状态值
    let params = ListEventsParams {
        from: None,
        to: None,
        status: Some("invalid_status".to_string()),
        keyword: None,
    };

    let result = mcp.list_events(Parameters(params)).await;
    assert!(result.is_err());
}
