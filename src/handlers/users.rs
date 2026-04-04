use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::Serialize;

use crate::db::repositories::UserRepository;
use crate::error::AppResult;
use crate::handlers::{require_admin, AuthenticatedUser};
use crate::models::{CreateUser, UpdateUser, User};

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub username: String,
    pub api_key: String,
    pub is_admin: bool,
    pub created_at: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            api_key: user.api_key,
            is_admin: user.is_admin,
            created_at: user.created_at,
        }
    }
}

pub async fn create_user(
    State(repo): State<UserRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
    Json(input): Json<CreateUser>,
) -> AppResult<Json<UserResponse>> {
    require_admin(&auth)?;

    let user = repo.create(input).await?;
    Ok(Json(UserResponse::from(user)))
}

pub async fn list_users(
    State(repo): State<UserRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
) -> AppResult<Json<Vec<UserResponse>>> {
    require_admin(&auth)?;

    let users = repo.list().await?;
    let response = users.into_iter().map(UserResponse::from).collect();
    Ok(Json(response))
}

pub async fn get_user(
    State(repo): State<UserRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> AppResult<Json<UserResponse>> {
    require_admin(&auth)?;

    let user = repo.find_by_id(&id).await?;
    Ok(Json(UserResponse::from(user)))
}

pub async fn update_user(
    State(repo): State<UserRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
    Json(input): Json<UpdateUser>,
) -> AppResult<Json<UserResponse>> {
    require_admin(&auth)?;

    let user = repo.update(&id, input).await?;
    Ok(Json(UserResponse::from(user)))
}

pub async fn delete_user(
    State(repo): State<UserRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    require_admin(&auth)?;

    repo.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
