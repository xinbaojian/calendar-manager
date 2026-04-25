use calendarsync::db::repositories::{EventRepository, UserRepository};
use calendarsync::db::{create_pool, run_migrations};
use calendarsync::handlers::{hash_password, AuthenticatedUser};
use calendarsync::mcp::{CalendarMCP, CreateEventParams, UpdateEventParams};
use rmcp::handler::server::wrapper::Parameters;
use std::sync::Arc;

#[tokio::test]
async fn test_mcp_update_event() {
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

    // 先创建一个测试日程
    let event_id = mcp
        .create_event(Parameters(CreateEventParams {
            title: "原始会议".to_string(),
            description: Some("原始描述".to_string()),
            location: Some("会议室A".to_string()),
            start_time: "2026-04-25T10:00:00+08:00".to_string(),
            end_time: "2026-04-25T11:00:00+08:00".to_string(),
            recurrence_rule: None,
            recurrence_until: None,
            reminder_minutes: Some(15),
            tags: Some(vec!["work".to_string()]),
        }))
        .await
        .unwrap();

    // 测试更新日程
    let params = UpdateEventParams {
        id: event_id.clone(),
        title: Some("更新后的会议".to_string()),
        description: Some("更新后的描述".to_string()),
        location: Some("会议室B".to_string()),
        start_time: None,
        end_time: None,
        status: None,
        recurrence_rule: None,
        recurrence_until: None,
        reminder_minutes: Some(30),
        tags: Some(vec!["work".to_string(), "important".to_string()]),
    };

    let result = mcp.update_event(Parameters(params)).await;
    assert!(result.is_ok());

    let event_json = result.unwrap();
    let event: serde_json::Value = serde_json::from_str(&event_json).unwrap();
    assert_eq!(event["title"], "更新后的会议");
    assert_eq!(event["description"], "更新后的描述");
    assert_eq!(event["location"], "会议室B");
    assert_eq!(event["reminder_minutes"], 30);
}

#[tokio::test]
async fn test_mcp_update_event_with_time() {
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

    // 先创建一个测试日程
    let event_id = mcp
        .create_event(Parameters(CreateEventParams {
            title: "原始会议".to_string(),
            description: None,
            location: None,
            start_time: "2026-04-25T10:00:00+08:00".to_string(),
            end_time: "2026-04-25T11:00:00+08:00".to_string(),
            recurrence_rule: None,
            recurrence_until: None,
            reminder_minutes: None,
            tags: None,
        }))
        .await
        .unwrap();

    // 测试更新日程时间
    let params = UpdateEventParams {
        id: event_id.clone(),
        title: None,
        description: None,
        location: None,
        start_time: Some("2026-04-25T14:00:00+08:00".to_string()),
        end_time: Some("2026-04-25T15:00:00+08:00".to_string()),
        status: None,
        recurrence_rule: None,
        recurrence_until: None,
        reminder_minutes: None,
        tags: None,
    };

    let result = mcp.update_event(Parameters(params)).await;
    assert!(result.is_ok());

    let event_json = result.unwrap();
    let event: serde_json::Value = serde_json::from_str(&event_json).unwrap();
    assert!(event["start_time"].as_str().unwrap().contains("14:00:00"));
    assert!(event["end_time"].as_str().unwrap().contains("15:00:00"));
}

#[tokio::test]
async fn test_mcp_update_event_not_found() {
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

    // 测试更新不存在的日程
    let params = UpdateEventParams {
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

    let result = mcp.update_event(Parameters(params)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mcp_update_event_access_denied() {
    // 使用内存数据库
    let test_id = uuid::Uuid::new_v4();
    let db_path = format!("sqlite::file:///tmp/mcp-test-{}.db", test_id);
    let pool = create_pool(&db_path).await.unwrap();
    run_migrations(&pool).await.unwrap();

    let user_repo = Arc::new(UserRepository::new(pool.clone()));

    let _webhook_repo = Arc::new(calendarsync::db::repositories::WebhookRepository::new(
        pool.clone(),
    ));
    let event_repo = Arc::new(EventRepository::new(pool.clone()));

    // 创建两个测试用户
    let password_hash = hash_password("test_password").unwrap();
    let user1 = user_repo
        .create(
            calendarsync::models::CreateUser {
                username: "user1".to_string(),
                password: None,
                is_admin: Some(false),
            },
            Some(password_hash.clone()),
        )
        .await
        .unwrap();

    let user2 = user_repo
        .create(
            calendarsync::models::CreateUser {
                username: "user2".to_string(),
                password: None,
                is_admin: Some(false),
            },
            Some(password_hash),
        )
        .await
        .unwrap();

    // 使用 user1 创建日程
    let auth_user1 = AuthenticatedUser {
        user_id: user1.id.clone(),
        is_admin: user1.is_admin,
    };
    let mcp1 = CalendarMCP::new(
        event_repo.clone(),
        None,
        auth_user1,
    );

    let event_id = mcp1
        .create_event(Parameters(CreateEventParams {
            title: "用户1的会议".to_string(),
            description: None,
            location: None,
            start_time: "2026-04-25T10:00:00+08:00".to_string(),
            end_time: "2026-04-25T11:00:00+08:00".to_string(),
            recurrence_rule: None,
            recurrence_until: None,
            reminder_minutes: None,
            tags: None,
        }))
        .await
        .unwrap();

    // 使用 user2 尝试更新 user1 的日程
    let auth_user2 = AuthenticatedUser {
        user_id: user2.id.clone(),
        is_admin: user2.is_admin,
    };
    let mcp2 = CalendarMCP::new(event_repo, None, auth_user2);

    let params = UpdateEventParams {
        id: event_id,
        title: Some("尝试修改".to_string()),
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

    let result = mcp2.update_event(Parameters(params)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mcp_update_event_empty_id() {
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

    // 测试空 ID
    let params = UpdateEventParams {
        id: "".to_string(),
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

    let result = mcp.update_event(Parameters(params)).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_mcp_update_event_invalid_time() {
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

    // 先创建一个测试日程
    let event_id = mcp
        .create_event(Parameters(CreateEventParams {
            title: "原始会议".to_string(),
            description: None,
            location: None,
            start_time: "2026-04-25T10:00:00+08:00".to_string(),
            end_time: "2026-04-25T11:00:00+08:00".to_string(),
            recurrence_rule: None,
            recurrence_until: None,
            reminder_minutes: None,
            tags: None,
        }))
        .await
        .unwrap();

    // 测试无效的时间格式
    let params = UpdateEventParams {
        id: event_id,
        title: None,
        description: None,
        location: None,
        start_time: Some("invalid-time".to_string()),
        end_time: None,
        status: None,
        recurrence_rule: None,
        recurrence_until: None,
        reminder_minutes: None,
        tags: None,
    };

    let result = mcp.update_event(Parameters(params)).await;
    assert!(result.is_err());
}
