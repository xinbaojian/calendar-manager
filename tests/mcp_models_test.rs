use calendarsync::mcp::models::*;

#[test]
fn test_create_event_params_schema() {
    // 验证结构体可以创建并序列化
    let _params = CreateEventParams {
        title: "测试".to_string(),
        description: None,
        location: None,
        start_time: "2026-04-25T10:00:00+08:00".to_string(),
        end_time: "2026-04-25T11:00:00+08:00".to_string(),
        recurrence_rule: None,
        recurrence_until: None,
        reminder_minutes: None,
        tags: None,
    };

    // 验证 JSON Schema 可以生成
    let _schema = schemars::schema_for!(CreateEventParams);
}

#[test]
fn test_create_event_params_serialization() {
    let params = CreateEventParams {
        title: "测试会议".to_string(),
        description: Some("讨论 MCP 集成".to_string()),
        location: Some("会议室 A".to_string()),
        start_time: "2026-04-25T10:00:00+08:00".to_string(),
        end_time: "2026-04-25T11:00:00+08:00".to_string(),
        recurrence_rule: None,
        recurrence_until: None,
        reminder_minutes: Some(15),
        tags: Some(vec!["work".to_string(), "important".to_string()]),
    };

    let json = serde_json::to_string(&params).unwrap();
    assert!(json.contains("\"title\":\"测试会议\""));
    assert!(json.contains("\"reminder_minutes\":15"));
}

#[test]
fn test_list_events_params_schema() {
    let _params = ListEventsParams {
        from: None,
        to: None,
        status: None,
        keyword: None,
    };

    // 验证 JSON Schema 可以生成
    let _schema = schemars::schema_for!(ListEventsParams);
}

#[test]
fn test_list_events_params_with_filters() {
    let params = ListEventsParams {
        from: Some("2026-04-01T00:00:00+08:00".to_string()),
        to: Some("2026-04-30T23:59:59+08:00".to_string()),
        status: Some("active".to_string()),
        keyword: Some("会议".to_string()),
    };

    let json = serde_json::to_string(&params).unwrap();
    assert!(json.contains("\"status\":\"active\""));
    assert!(json.contains("\"keyword\":\"会议\""));
}

#[test]
fn test_get_event_params_schema() {
    let _params = GetEventParams {
        id: "event-123".to_string(),
    };

    // 验证 JSON Schema 可以生成
    let _schema = schemars::schema_for!(GetEventParams);
}

#[test]
fn test_update_event_params_schema() {
    let _params = UpdateEventParams {
        id: "event-123".to_string(),
        title: None,
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

    // 验证 JSON Schema 可以生成
    let _schema = schemars::schema_for!(UpdateEventParams);
}

#[test]
fn test_update_event_params_partial_update() {
    let params = UpdateEventParams {
        id: "event-123".to_string(),
        title: Some("更新后的标题".to_string()),
        description: None,
        location: None,
        start_time: None,
        end_time: None,
        status: Some("active".to_string()),
        recurrence_rule: None,
        recurrence_until: None,
        reminder_minutes: None,
        tags: None,
    };

    let json = serde_json::to_string(&params).unwrap();
    assert!(json.contains("\"id\":\"event-123\""));
    assert!(json.contains("\"title\":\"更新后的标题\""));
    assert!(json.contains("\"status\":\"active\""));
}

#[test]
fn test_delete_event_params_schema() {
    let _params = DeleteEventParams {
        id: "event-to-delete".to_string(),
    };

    // 验证 JSON Schema 可以生成
    let _schema = schemars::schema_for!(DeleteEventParams);
}

#[test]
fn test_delete_event_params_serialization() {
    let params = DeleteEventParams {
        id: "event-to-delete".to_string(),
    };

    let json = serde_json::to_string(&params).unwrap();
    assert!(json.contains("\"id\":\"event-to-delete\""));
}
