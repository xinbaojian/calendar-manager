use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Request, State},
    http::HeaderMap,
    response::Response,
    Router,
};
use http_body_util::BodyExt;
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
};
use tower_service::Service;

use crate::error::AppError;
use crate::handlers::AuthenticatedUser;
use crate::mcp::server::CalendarMCP;
use crate::state::AppState;

/// 创建 MCP HTTP 路由
pub fn create_mcp_router() -> Router<AppState> {
    Router::new()
        .route("/mcp", axum::routing::any(mcp_handler))
        .route("/mcp/", axum::routing::any(mcp_handler))
}

/// MCP HTTP 处理器
///
/// 使用 rmcp 的 StreamableHttpService 处理 JSON-RPC 2.0 消息，
/// 每个请求独立认证（无状态模式），与 REST API 行为一致。
async fn mcp_handler(State(state): State<AppState>, req: Request) -> Result<Response, AppError> {
    let auth_user = extract_api_key(req.headers(), &state).await?;

    let mcp_service = CalendarMCP::new(
        state.event_repo.clone(),
        Some((*state.webhook_service).clone()),
        auth_user,
    );

    let session_manager = Arc::new(LocalSessionManager::default());
    let mut config = StreamableHttpServerConfig::default();
    config.stateful_mode = false;
    // 允许所有 Host 头（Nginx 反向代理场景下 Host 为外部域名，不在默认白名单中）
    config = config.disable_allowed_hosts();

    let streamable_service =
        StreamableHttpService::new(move || Ok(mcp_service.clone()), session_manager, config);

    let mut service = streamable_service;
    let response = service
        .call(req)
        .await
        .map_err(|e| AppError::Internal(format!("MCP service error: {:?}", e)))?;

    let (parts, body) = response.into_parts();
    let collected_body = body
        .collect()
        .await
        .map_err(|e| AppError::Internal(format!("Failed to collect response body: {:?}", e)))?
        .to_bytes();

    Ok(Response::from_parts(parts, Body::from(collected_body)))
}

/// 从请求头提取并验证 API Key
async fn extract_api_key(
    headers: &HeaderMap,
    state: &AppState,
) -> Result<AuthenticatedUser, AppError> {
    let api_key = headers
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::InvalidApiKey)?;

    let user = state
        .user_repo
        .find_by_api_key(api_key)
        .await
        .map_err(|_| AppError::InvalidApiKey)?;

    Ok(AuthenticatedUser {
        user_id: user.id,
        is_admin: user.is_admin,
    })
}
