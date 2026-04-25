use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::Serialize;
use uuid::Uuid;

use crate::error::AppResult;
use crate::handlers::{hash_password, require_admin, AuthenticatedUser};
use crate::models::{CreateUser, UpdateUser, User};
use crate::state::AppState;

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub is_admin: bool,
    pub created_at: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            is_admin: user.is_admin,
            created_at: user.created_at,
        }
    }
}

#[derive(Serialize)]
pub struct CreatedUserResponse {
    #[serde(flatten)]
    pub user: UserResponse,
    pub api_key: String,
}

pub async fn create_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Json(input): Json<CreateUser>,
) -> AppResult<(StatusCode, Json<CreatedUserResponse>)> {
    require_admin(&auth)?;

    let password_hash = match input.password.as_ref() {
        Some(pwd) if !pwd.is_empty() => {
            let pwd = pwd.clone();
            Some(
                tokio::task::spawn_blocking(move || hash_password(&pwd))
                    .await
                    .map_err(|e| crate::error::AppError::Internal(e.to_string()))??,
            )
        }
        _ => None,
    };

    let user = state.user_repo.create(input, password_hash).await?;
    let api_key = user.api_key.clone();
    Ok((
        StatusCode::CREATED,
        Json(CreatedUserResponse {
            user: UserResponse::from(user),
            api_key,
        }),
    ))
}

pub async fn list_users(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
) -> AppResult<Json<Vec<UserResponse>>> {
    require_admin(&auth)?;

    let users = state.user_repo.list().await?;
    let response = users.into_iter().map(UserResponse::from).collect();
    Ok(Json(response))
}

pub async fn get_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> AppResult<Json<UserResponse>> {
    require_admin(&auth)?;

    let user = state.user_repo.find_by_id(&id).await?;
    Ok(Json(UserResponse::from(user)))
}

pub async fn update_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
    Json(input): Json<UpdateUser>,
) -> AppResult<Json<UserResponse>> {
    require_admin(&auth)?;

    let user = state.user_repo.update(&id, input).await?;
    Ok(Json(UserResponse::from(user)))
}

pub async fn delete_user(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    require_admin(&auth)?;

    state.user_repo.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct ApiKeyResponse {
    pub api_key: String,
}

/// 获取当前用户的 API Key
pub async fn get_api_key(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
) -> AppResult<Json<ApiKeyResponse>> {
    let user = state.user_repo.find_by_id(&auth.user_id).await?;
    Ok(Json(ApiKeyResponse {
        api_key: user.api_key,
    }))
}

/// 重新生成当前用户的 API Key
pub async fn regenerate_api_key(
    State(state): State<AppState>,
    Extension(auth): Extension<AuthenticatedUser>,
) -> AppResult<Json<ApiKeyResponse>> {
    let new_api_key = Uuid::new_v4().to_string();
    state
        .user_repo
        .update_api_key(&auth.user_id, &new_api_key)
        .await?;
    Ok(Json(ApiKeyResponse {
        api_key: new_api_key,
    }))
}
