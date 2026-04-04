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
use calendarsync::state::AppState;

async fn create_test_app() -> AppState {
    let pool = create_pool("sqlite::memory:").await.unwrap();
    run_migrations(&pool).await.unwrap();

    AppState {
        user_repo: std::sync::Arc::new(UserRepository::new(pool.clone())),
        event_repo: std::sync::Arc::new(EventRepository::new(pool.clone())),
        webhook_repo: std::sync::Arc::new(WebhookRepository::new(pool)),
    }
}

fn app_with_state(state: AppState) -> axum::Router {
    use axum::middleware;
    use axum::routing::{get, post};
    use tower_http::cors::CorsLayer;

    let public_routes = axum::Router::new()
        .route(
            "/calendar/:user_id/subscribe.ics",
            get(subscribe_calendar),
        );

    let api_routes = axum::Router::new()
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
            state.user_repo.clone(),
            auth_middleware,
        ));

    axum::Router::new()
        .merge(public_routes)
        .merge(api_routes)
        .with_state(state.clone())
        .layer(CorsLayer::permissive())
}

#[tokio::test]
async fn test_create_user_returns_201_and_api_key() {
    let state = create_test_app().await;
    let admin = state.user_repo.create(calendarsync::models::CreateUser {
        username: "admin".to_string(),
        is_admin: Some(true),
    }).await.unwrap();

    let app = app_with_state(state);

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/users")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-API-Key", &admin.api_key)
        .body(Body::from(
            json!({"username": "testuser", "is_admin": false}).to_string(),
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
    let admin = state.user_repo.create(calendarsync::models::CreateUser {
        username: "admin".to_string(),
        is_admin: Some(true),
    }).await.unwrap();

    let app = app_with_state(state);

    let req = Request::builder()
        .method(Method::GET)
        .uri("/api/users")
        .header("X-API-Key", &admin.api_key)
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let users: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let user = &users[0];
    // api_key should NOT be in list response
    assert!(user.get("api_key").is_none(), "api_key should be hidden in list response");
}

#[tokio::test]
async fn test_create_event_returns_201() {
    let state = create_test_app().await;
    let user = state.user_repo.create(calendarsync::models::CreateUser {
        username: "testuser".to_string(),
        is_admin: Some(false),
    }).await.unwrap();

    let app = app_with_state(state);

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/events")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-API-Key", &user.api_key)
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
    let user = state.user_repo.create(calendarsync::models::CreateUser {
        username: "testuser".to_string(),
        is_admin: Some(false),
    }).await.unwrap();

    let app = app_with_state(state);

    let req = Request::builder()
        .method(Method::POST)
        .uri("/api/webhooks")
        .header(header::CONTENT_TYPE, "application/json")
        .header("X-API-Key", &user.api_key)
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
    let user = state.user_repo.create(calendarsync::models::CreateUser {
        username: "caluser".to_string(),
        is_admin: Some(false),
    }).await.unwrap();

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
