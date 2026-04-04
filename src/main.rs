use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use calendarsync::config::load_config;
use calendarsync::db::repositories::{EventRepository, UserRepository, WebhookRepository};
use calendarsync::db::{create_pool, run_migrations};
use calendarsync::handlers::auth_middleware;
use calendarsync::handlers::calendar::subscribe_calendar;
use calendarsync::handlers::events::{create_event, delete_event, get_event, list_events, search_events, update_event};
use calendarsync::handlers::users::{create_user, delete_user, get_user, list_users, update_user};
use calendarsync::handlers::webhooks::{create_webhook, delete_webhook, get_webhook, list_webhooks, update_webhook};
use calendarsync::state::AppState;
use calendarsync::templates::IndexTemplate;

async fn index_handler() -> IndexTemplate {
    IndexTemplate
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "calendarsync=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = load_config(Path::new("config.toml"))?;

    let pool = create_pool(&format!("sqlite:{}", config.database.path)).await?;
    run_migrations(&pool).await?;

    let state = AppState {
        user_repo: Arc::new(UserRepository::new(pool.clone())),
        event_repo: Arc::new(EventRepository::new(pool.clone())),
        webhook_repo: Arc::new(WebhookRepository::new(pool)),
    };

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/", get(index_handler))
        .route("/events", get(index_handler))
        .route("/settings", get(index_handler))
        .route(
            "/calendar/:user_id/subscribe.ics",
            get(subscribe_calendar),
        );

    // API routes (auth required)
    let api_routes = Router::new()
        .route(
            "/api/users",
            post(create_user).get(list_users),
        )
        .route(
            "/api/users/:id",
            get(get_user).put(update_user).delete(delete_user),
        )
        .route(
            "/api/events",
            post(create_event).get(list_events),
        )
        .route("/api/events/search", get(search_events))
        .route(
            "/api/events/:id",
            get(get_event).put(update_event).delete(delete_event),
        )
        .route(
            "/api/webhooks",
            post(create_webhook).get(list_webhooks),
        )
        .route(
            "/api/webhooks/:id",
            get(get_webhook).put(update_webhook).delete(delete_webhook),
        )
        .layer(middleware::from_fn_with_state(
            state.user_repo.clone(),
            auth_middleware,
        ));

    let app = Router::new()
        .merge(public_routes)
        .merge(api_routes)
        .with_state(state.clone())
        .layer(CorsLayer::permissive());

    // Scheduled cleanup task
    let event_repo_cleanup = state.event_repo.clone();
    let cleanup_interval = config.cleanup.check_interval_hours;
    let auto_delete_days = config.cleanup.auto_delete_expired_days;

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(cleanup_interval * 3600));

        loop {
            interval.tick().await;

            let now = chrono::Utc::now().to_rfc3339();
            match event_repo_cleanup.mark_expired(&now).await {
                Ok(count) => {
                    if count > 0 {
                        tracing::info!("Marked {} events as expired", count);
                    }
                }
                Err(e) => tracing::error!("Failed to mark expired events: {}", e),
            }

            if auto_delete_days > 0 {
                match event_repo_cleanup
                    .delete_old_expired(auto_delete_days as i64)
                    .await
                {
                    Ok(count) => {
                        if count > 0 {
                            tracing::info!("Deleted {} old expired events", count);
                        }
                    }
                    Err(e) => tracing::error!("Failed to delete old events: {}", e),
                }
            }
        }
    });

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("CalendarSync listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
