use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

use calendarsync::db::repositories::{EventRepository, UserRepository, WebhookRepository};
use calendarsync::db::{create_pool, run_migrations};
use calendarsync::handlers::auth_middleware;
use calendarsync::handlers::calendar::subscribe_calendar;
use calendarsync::handlers::events::{create_event, delete_event, get_event, list_events, update_event};
use calendarsync::handlers::users::{create_user, delete_user, get_user, list_users, update_user};
use calendarsync::handlers::webhooks::{create_webhook, delete_webhook, get_webhook, list_webhooks, update_webhook};
use calendarsync::handlers::login;
use calendarsync::state::AppState;
use calendarsync::services::WebhookService;

const TEST_JWT_SECRET: &str = "test-jwt-secret-for-testing";
const TEST_JWT_EXP_HOURS: u64 = 24;

async fn create_test_app() -> AppState {
    // 为每个测试使用唯一的数据库文件
    let test_id = uuid::Uuid::new_v4();
    let db_path = format!("/tmp/calendarsync-test-{}.db", test_id);
    let pool = create_pool(&format!("sqlite::{}", db_path)).await.unwrap();
    run_migrations(&pool).await.unwrap();

    AppState {
        user_repo: std::sync::Arc::new(UserRepository::new(pool.clone())),
        event_repo: std::sync::Arc::new(EventRepository::new(pool.clone())),
        webhook_repo: std::sync::Arc::new(WebhookRepository::new(pool.clone())),
        webhook_service: std::sync::Arc::new(WebhookService::new(
            WebhookRepository::new(pool),
            10,
            3,
        )),
        jwt_secret: TEST_JWT_SECRET.to_string(),
        jwt_exp_hours: TEST_JWT_EXP_HOURS,
    }
}

/// Create a test user with both API key and password set, return (user, api_key)
async fn create_test_user(state: &AppState, username: &str, is_admin: bool) -> (calendarsync::models::User, String) {
    let password_hash = calendarsync::handlers::hash_password("testpass123").unwrap();
    let user = state.user_repo.create(calendarsync::models::CreateUser {
        username: username.to_string(),
        password: None,
        is_admin: Some(is_admin),
    }, Some(password_hash)).await.unwrap();
    let api_key = user.api_key.clone();
    (user, api_key)
}

fn app_with_state(state: AppState) -> axum::Router {
    use axum::middleware;
    use axum::routing::{get, post};
    use calendarsync::handlers::change_password;
    use tower_http::cors::CorsLayer;

    let public_routes = axum::Router::new()
        .route(
            "/calendar/:user_id/subscribe.ics",
            get(subscribe_calendar),
        )
        .route("/api/auth/login", post(login));

    let api_routes = axum::Router::new()
        .route(
            "/api/auth/change-password",
            post(change_password),
        )
        .route(
            "/api/users",
            post(create_user).get(list_users),
        )
        .route(
            "/api/users/:id",
            get(get_user)
                .put(update_user)
                .delete(delete_user),
        )
        .route(
            "/api/events",
            post(create_event).get(list_events),
        )
        .route(
            "/api/events/:id",
            get(get_event)
                .put(update_event)
                .delete(delete_event),
        )
        .route(
            "/api/webhooks",
            post(create_webhook).get(list_webhooks),
        )
        .route(
            "/api/webhooks/:id",
            get(get_webhook)
                .put(update_webhook)
                .delete(delete_webhook),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    axum::Router::new()
        .merge(public_routes)
        .merge(api_routes)
        .with_state(state.clone())
        .layer(CorsLayer::permissive())
}

// ===== API Key Authentication Tests =====

#[tokio::test]
async fn test_create_user_returns_201_and_api_key() {
    let state = create_test_app().await;
    let (_, admin_api_key) = create_test_user(&state, "admin", true).await;

    let app = app_with_state(state);

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/users")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-API-Key", &admin_api_key)
        .body(Body::from(
            json!({"username": "testuser", "password": "newpass123", "is_admin": false}).to_string(),
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let user: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(user["username"], "testuser");
    assert_eq!(user["is_admin"], false);
    assert!(!user["api_key"].as_str().unwrap_or("").is_empty());
}

#[tokio::test]
async fn test_list_users_hides_api_key() {
    let state = create_test_app().await;
    let (_, admin_api_key) = create_test_user(&state, "admin", true).await;

    let app = app_with_state(state);

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/users")
        .header("X-API-Key", &admin_api_key)
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let users: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let user = &users[0];
    // api_key should NOT be in list response
    assert!(user.get("api_key").is_none(), "api_key should be hidden in list response");
    // password_hash should also NOT be in response
    assert!(user.get("password_hash").is_none(), "password_hash should be hidden in list response");
}

#[tokio::test]
async fn test_create_event_returns_201() {
    let state = create_test_app().await;
    let (user, api_key) = create_test_user(&state, "testuser", false).await;

    let app = app_with_state(state);

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/events")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-API-Key", &api_key)
        .body(Body::from(
            json!({
                "user_id": user.id,
                "event": {
                    "title": "Meeting",
                    "description": "Team standup",
                    "start_time": "2026-04-10T09:00:00Z",
                    "end_time": "2026-04-10T10:00:00Z"
                }
            }).to_string(),
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let event: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(event["title"], "Meeting");
}

#[tokio::test]
async fn test_create_webhook_returns_201() {
    let state = create_test_app().await;
    let (user, api_key) = create_test_user(&state, "testuser", false).await;

    let app = app_with_state(state);

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/webhooks")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-API-Key", &api_key)
        .body(Body::from(
            json!({
                "user_id": user.id,
                "webhook": {
                    "url": "https://example.com/webhook",
                    "events": ["event.created"]
                }
            }).to_string(),
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_calendar_subscription() {
    let state = create_test_app().await;
    let (user, _) = create_test_user(&state, "caluser", false).await;

    let app = app_with_state(state);

    let req = Request::builder()
        .method(Method::GET)
        .uri(&format!("/calendar/{}/subscribe.ics", user.id))
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let ical = String::from_utf8(body.to_vec()).unwrap();
    assert!(ical.contains("BEGIN:VCALENDAR"));
}

#[tokio::test]
async fn test_unauthorized_access() {
    let state = create_test_app().await;
    let app = app_with_state(state);

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/users")
        .header("X-API-Key", "invalid-key")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_create_event_with_empty_title_returns_400() {
    let state = create_test_app().await;
    let (user, api_key) = create_test_user(&state, "testuser", false).await;

    let app = app_with_state(state);

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/events")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-API-Key", &api_key)
        .body(Body::from(
            json!({
                "user_id": user.id,
                "event": {
                    "title": "   ",
                    "description": "Empty title event",
                    "start_time": "2026-04-10T09:00:00Z",
                    "end_time": "2026-04-10T10:00:00Z"
                }
            }).to_string(),
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_event_with_invalid_dates_returns_400() {
    let state = create_test_app().await;
    let (user, api_key) = create_test_user(&state, "testuser", false).await;

    let app = app_with_state(state);

    // Test: end_time before start_time
    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/events")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-API-Key", &api_key)
        .body(Body::from(
            json!({
                "user_id": user.id,
                "event": {
                    "title": "Invalid Time Event",
                    "start_time": "2026-04-10T10:00:00Z",
                    "end_time": "2026-04-10T09:00:00Z"
                }
            }).to_string(),
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_create_webhook_with_invalid_url_returns_400() {
    let state = create_test_app().await;
    let (user, api_key) = create_test_user(&state, "testuser", false).await;

    let app = app_with_state(state);

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/webhooks")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-API-Key", &api_key)
        .body(Body::from(
            json!({
                "user_id": user.id,
                "webhook": {
                    "url": "not-a-valid-url",
                    "events": ["event.created"]
                }
            }).to_string(),
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_search_event_with_keyword() {
    let state = create_test_app().await;
    let (user, api_key) = create_test_user(&state, "testuser", false).await;

    // Create an event
    let _event = state.event_repo.create(
        user.id.clone(),
        calendarsync::models::CreateEvent {
            title: "Team Meeting".to_string(),
            description: Some("Weekly standup".to_string()),
            location: None,
            start_time: "2026-04-10T09:00:00Z".to_string(),
            end_time: "2026-04-10T10:00:00Z".to_string(),
            recurrence_rule: None,
            recurrence_until: None,
            reminder_minutes: None,
            tags: None,
        },
    ).await.unwrap();

    let app = app_with_state(state);

    // Search for "Team" keyword
    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/events?keyword=Team")
        .header("X-API-Key", &api_key)
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let events: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(events.as_array().unwrap().len(), 1);
    assert_eq!(events[0]["title"], "Team Meeting");
}

// ===== JWT Authentication Tests =====

#[tokio::test]
async fn test_login_returns_token() {
    let state = create_test_app().await;
    create_test_user(&state, "logintest", false).await;

    let app = app_with_state(state);

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({"username": "logintest", "password": "testpass123"}).to_string(),
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(!data["token"].as_str().unwrap_or("").is_empty());
    assert_eq!(data["user"]["username"], "logintest");
    assert_eq!(data["user"]["is_admin"], false);
}

#[tokio::test]
async fn test_login_with_wrong_password_returns_401() {
    let state = create_test_app().await;
    create_test_user(&state, "logintest2", false).await;

    let app = app_with_state(state);

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({"username": "logintest2", "password": "wrongpassword"}).to_string(),
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_login_with_nonexistent_user_returns_401() {
    let state = create_test_app().await;
    let app = app_with_state(state);

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({"username": "nonexistent", "password": "anypass"}).to_string(),
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_jwt_auth_allows_api_access() {
    let state = create_test_app().await;
    let (_user, _) = create_test_user(&state, "jwtauth", false).await;

    let app = app_with_state(state);

    // Login to get token
    let login_req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({"username": "jwtauth", "password": "testpass123"}).to_string(),
        ))
        .unwrap();

    let login_resp = app.clone().oneshot(login_req).await.unwrap();
    let login_body = login_resp.into_body().collect().await.unwrap().to_bytes();
    let token = serde_json::from_slice::<serde_json::Value>(&login_body).unwrap()["token"]
        .as_str().unwrap().to_string();

    // Use JWT token to list events
    let list_req = Request::builder()
        .method(Method::GET)
        .uri("/api/events")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let list_resp = app.oneshot(list_req).await.unwrap();
    assert_eq!(list_resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_change_password() {
    let state = create_test_app().await;
    let (_, api_key) = create_test_user(&state, "changepw", false).await;

    let app = app_with_state(state);

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/change-password")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-API-Key", &api_key)
        .body(Body::from(
            json!({"current_password": "testpass123", "new_password": "newpass456"}).to_string(),
        ))
        .unwrap();

    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(data["message"], "密码修改成功");

    // Verify can login with new password
    let login_req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({"username": "changepw", "password": "newpass456"}).to_string(),
        ))
        .unwrap();

    let login_resp = app.oneshot(login_req).await.unwrap();
    assert_eq!(login_resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_change_password_with_wrong_current_returns_401() {
    let state = create_test_app().await;
    let (_, api_key) = create_test_user(&state, "changepw2", false).await;

    let app = app_with_state(state);

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/change-password")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-API-Key", &api_key)
        .body(Body::from(
            json!({"current_password": "wrongpass", "new_password": "newpass456"}).to_string(),
        ))
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
