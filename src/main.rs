use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use chrono_tz::Asia::Shanghai;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use calendarsync::config::load_config;
use calendarsync::db::repositories::{EventRepository, UserRepository, WebhookRepository};
use calendarsync::db::{create_pool, run_migrations};
use calendarsync::handlers::calendar::subscribe_calendar;
use calendarsync::handlers::events::{
    create_event, delete_event, get_event, list_events, update_event,
};
use calendarsync::handlers::users::{
    create_user, delete_user, get_api_key, get_user, list_users, regenerate_api_key, update_user,
};
use calendarsync::handlers::webhooks::{
    create_webhook, delete_webhook, get_webhook, list_webhooks, update_webhook,
};
use calendarsync::handlers::{auth_middleware, change_password, hash_password, login};
use calendarsync::mcp;
use calendarsync::services::WebhookService;
use calendarsync::state::AppState;
use calendarsync::templates::IndexTemplate;

async fn index_handler() -> IndexTemplate {
    IndexTemplate
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "calendarsync=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = load_config(Path::new("config.toml"))?;

    let pool = create_pool(&format!("sqlite::{}", config.database.path)).await?;
    run_migrations(&pool).await?;

    let user_repo = Arc::new(UserRepository::new(pool.clone()));

    // 初始化管理员用户（如果不存在）
    let admin_api_key = config.auth.admin_api_key.clone();
    let jwt_secret = config.auth.jwt_secret.clone();
    let jwt_exp_hours = config.auth.jwt_exp_hours;
    let admin_password = config.auth.admin_password.clone();

    match user_repo.find_by_api_key(&admin_api_key).await {
        Ok(_) => tracing::info!("Admin user already exists"),
        Err(_) => {
            tracing::info!("Creating admin user...");
            let admin_password_hash =
                tokio::task::spawn_blocking(move || hash_password(&admin_password)).await??;

            let admin_username = config.auth.admin_username.clone();
            let admin = user_repo
                .create(
                    calendarsync::models::CreateUser {
                        username: admin_username,
                        password: None,
                        is_admin: Some(true),
                    },
                    Some(admin_password_hash),
                )
                .await?;
            // 更新管理员的 api_key 为配置文件中的密钥
            user_repo.update_api_key(&admin.id, &admin_api_key).await?;
            tracing::info!("Admin user created with configured API key and password");
        }
    }

    let state = AppState {
        user_repo,
        event_repo: Arc::new(EventRepository::new(pool.clone())),
        webhook_repo: Arc::new(WebhookRepository::new(pool.clone())),
        webhook_service: Arc::new(WebhookService::new(
            WebhookRepository::new(pool),
            config.webhook.timeout_seconds,
            config.webhook.max_retries,
        )),
        jwt_secret,
        jwt_exp_hours,
    };

    // SPA fallback — serve index.html for all frontend routes
    let spa_routes = Router::new()
        .route("/events", get(index_handler))
        .route("/settings", get(index_handler))
        .route("/webhooks", get(index_handler))
        .route("/users", get(index_handler));

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/", get(index_handler))
        .merge(spa_routes)
        .route("/calendar/:user_id/subscribe.ics", get(subscribe_calendar))
        .route("/api/auth/login", post(login))
        // 静态文件服务（仅 favicon）
        .nest_service("/static", ServeDir::new("static").precompressed_gzip())
        .route(
            "/favicon.ico",
            axum::routing::get(async || {
                // 重定向到 /static/favicon.ico
                axum::response::Redirect::permanent("/static/favicon.ico")
            }),
        );

    // API routes (auth required)
    let api_routes = Router::new()
        .route("/api/auth/change-password", post(change_password))
        .route(
            "/api/auth/api-key",
            get(get_api_key).post(regenerate_api_key),
        )
        .route("/api/users", post(create_user).get(list_users))
        .route(
            "/api/users/:id",
            get(get_user).put(update_user).delete(delete_user),
        )
        .route("/api/events", post(create_event).get(list_events))
        .route(
            "/api/events/:id",
            get(get_event).put(update_event).delete(delete_event),
        )
        .route("/api/webhooks", post(create_webhook).get(list_webhooks))
        .route(
            "/api/webhooks/:id",
            get(get_webhook).put(update_webhook).delete(delete_webhook),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // MCP routes (public, no auth required)
    let mcp_routes = mcp::create_mcp_router();

    let app = Router::new()
        .merge(public_routes)
        .merge(api_routes)
        .merge(mcp_routes)
        .with_state(state.clone())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    // Scheduled cleanup task
    let event_repo_cleanup = state.event_repo.clone();
    let cleanup_interval = config.cleanup.check_interval_hours;
    let auto_delete_days = config.cleanup.auto_delete_expired_days;

    tokio::spawn(async move {
        // Skip the immediate first tick
        let mut interval = tokio::time::interval(Duration::from_secs(cleanup_interval * 3600));
        interval.tick().await;

        loop {
            interval.tick().await;

            let now = chrono::Utc::now().with_timezone(&Shanghai).to_rfc3339();
            if let Err(e) = event_repo_cleanup.mark_expired(&now).await {
                tracing::error!(error = %e, "Failed to mark expired events");
            }

            if auto_delete_days > 0 {
                if let Err(e) = event_repo_cleanup
                    .delete_old_expired(auto_delete_days as i64)
                    .await
                {
                    tracing::error!(error = %e, "Failed to delete old events");
                }
            }
        }
    });

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("CalendarSync listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
}
