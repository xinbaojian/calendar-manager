use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use crate::{
    db::repositories::UserRepository,
    error::{AppError, AppResult},
    models::User,
};

#[derive(Clone)]
pub struct AuthenticatedUser {
    pub user: User,
    pub is_admin: bool,
}

pub async fn auth_middleware(
    State(user_repo): State<std::sync::Arc<UserRepository>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let api_key = headers
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok())
        .ok_or(AppError::InvalidApiKey)?;

    let user = user_repo.find_by_api_key(api_key).await?;

    request.extensions_mut().insert(AuthenticatedUser {
        is_admin: user.is_admin,
        user,
    });

    Ok(next.run(request).await)
}

pub fn require_admin(auth_user: &AuthenticatedUser) -> AppResult<()> {
    if !auth_user.is_admin {
        return Err(AppError::InsufficientPermissions);
    }
    Ok(())
}

pub fn check_user_access(auth_user: &AuthenticatedUser, target_user_id: &str) -> AppResult<()> {
    if auth_user.is_admin {
        return Ok(());
    }
    if auth_user.user.id == target_user_id {
        return Ok(());
    }
    Err(AppError::InsufficientPermissions)
}
