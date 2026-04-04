use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use axum::{
    extract::{Request, State},
    http::HeaderMap,
    middleware::Next,
    response::Response,
    Json,
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use password_hash::rand_core::OsRng;
use serde::{Deserialize, Serialize};
use chrono::Utc;

use crate::{
    error::{AppError, AppResult},
    state::AppState,
};
use crate::models::{LoginRequest, ChangePasswordRequest, LoginResponse, UserSummary};

#[derive(Clone)]
pub struct AuthenticatedUser {
    pub user_id: String,
    pub is_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub username: String,
    pub is_admin: bool,
    pub exp: usize,
    pub iat: usize,
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_user = if let Some(auth_header) = headers.get("Authorization") {
        // Bearer JWT token path
        let auth_str = auth_header.to_str().map_err(|_| AppError::InvalidToken)?;
        if !auth_str.starts_with("Bearer ") {
            return Err(AppError::InvalidToken);
        }
        let token = &auth_str[7..];
        let claims = decode_jwt(token, &state.jwt_secret)?;
        match state.user_repo.find_by_id(&claims.sub).await {
            Ok(user) => AuthenticatedUser {
                user_id: user.id,
                is_admin: user.is_admin,
            },
            Err(_) => return Err(AppError::InvalidToken),
        }
    } else if let Some(api_key) = headers.get("X-API-Key").and_then(|v| v.to_str().ok()) {
        // API Key path (legacy)
        let user = state.user_repo.find_by_api_key(api_key).await?;
        AuthenticatedUser {
            user_id: user.id,
            is_admin: user.is_admin,
        }
    } else {
        return Err(AppError::InvalidApiKey);
    };

    tracing::info!(user_id = %auth_user.user_id, "User authenticated");
    request.extensions_mut().insert(auth_user);

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
    if auth_user.user_id == target_user_id {
        return Ok(());
    }
    Err(AppError::InsufficientPermissions)
}

// --- JWT helpers ---

pub fn create_jwt(user_id: &str, username: &str, is_admin: bool, secret: &str, exp_hours: u64) -> AppResult<String> {
    let now = Utc::now();
    let claims = JwtClaims {
        sub: user_id.to_string(),
        username: username.to_string(),
        is_admin,
        iat: now.timestamp() as usize,
        exp: (now.timestamp() + (exp_hours as i64 * 3600)) as usize,
    };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|e| AppError::Internal(e.to_string()))
}

fn decode_jwt(token: &str, secret: &str) -> AppResult<JwtClaims> {
    let token_data = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    ).map_err(|_| AppError::InvalidToken)?;
    Ok(token_data.claims)
}

// --- Password helpers ---

pub fn hash_password(password: &str) -> AppResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

// --- Auth handlers ---

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<LoginResponse>> {
    let user = state.user_repo.find_by_username(&req.username).await
        .map_err(|_| AppError::InvalidCredentials)?;

    match &user.password_hash {
        Some(hash) => {
            let valid = tokio::task::spawn_blocking({
                let password = req.password.clone();
                let hash = hash.clone();
                move || verify_password(&password, &hash)
            })
            .await
            .map_err(|e| AppError::Internal(e.to_string()))??;

            if !valid {
                return Err(AppError::InvalidCredentials);
            }
        }
        None => return Err(AppError::PasswordRequired),
    }

    let token = create_jwt(&user.id, &user.username, user.is_admin, &state.jwt_secret, state.jwt_exp_hours)?;

    Ok(Json(LoginResponse {
        token,
        user: UserSummary::from(&user),
    }))
}

pub async fn change_password(
    State(state): State<AppState>,
    axum::Extension(auth): axum::Extension<AuthenticatedUser>,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let user = state.user_repo.find_by_id(&auth.user_id).await?;

    match &user.password_hash {
        Some(hash) => {
            let valid = tokio::task::spawn_blocking({
                let password = req.current_password.clone();
                let hash = hash.clone();
                move || verify_password(&password, &hash)
            })
            .await
            .map_err(|e| AppError::Internal(e.to_string()))??;

            if !valid {
                return Err(AppError::IncorrectPassword);
            }
        }
        None => return Err(AppError::PasswordRequired),
    }

    let new_hash = tokio::task::spawn_blocking({
        let password = req.new_password.clone();
        move || hash_password(&password)
    })
    .await
    .map_err(|e| AppError::Internal(e.to_string()))??;

    state.user_repo.update_password(&auth.user_id, &new_hash).await?;

    Ok(Json(serde_json::json!({ "message": "密码修改成功" })))
}
