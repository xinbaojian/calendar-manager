use chrono::Utc;
use chrono_tz::Asia::Shanghai;
use sqlx::{Pool, Sqlite};

use crate::error::{AppError, AppResult};
use crate::models::{CreateUser, UpdateUser, User};

#[derive(Clone)]
pub struct UserRepository {
    pool: Pool<Sqlite>,
}

impl UserRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        input: CreateUser,
        password_hash: Option<String>,
    ) -> AppResult<User> {
        let user = User::new(input.username, input.is_admin.unwrap_or(false));

        sqlx::query(
            "INSERT INTO users (id, username, api_key, password_hash, is_admin, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .bind(&user.id)
        .bind(&user.username)
        .bind(&user.api_key)
        .bind(&password_hash)
        .bind(user.is_admin as i32)
        .bind(&user.created_at)
        .bind(&user.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                AppError::DuplicateUsername(user.username.clone())
            } else {
                AppError::Database(e)
            }
        })?;

        let mut user = user;
        user.password_hash = password_hash;
        Ok(user)
    }

    pub async fn find_by_id(&self, id: &str) -> AppResult<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| AppError::UserNotFound(id.to_string()))
    }

    pub async fn find_by_api_key(&self, api_key: &str) -> AppResult<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE api_key = ?1")
            .bind(api_key)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| AppError::InvalidApiKey)
    }

    pub async fn find_by_username(&self, username: &str) -> AppResult<User> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?1")
            .bind(username)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| AppError::InvalidCredentials)
    }

    pub async fn list(&self) -> AppResult<Vec<User>> {
        sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn find_all(&self) -> AppResult<Vec<User>> {
        self.list().await
    }

    pub async fn update_api_key(&self, id: &str, api_key: &str) -> AppResult<()> {
        sqlx::query("UPDATE users SET api_key = ?1, updated_at = ?2 WHERE id = ?3")
            .bind(api_key)
            .bind(Utc::now().with_timezone(&Shanghai).to_rfc3339())
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;
        Ok(())
    }

    pub async fn update_password(&self, id: &str, password_hash: &str) -> AppResult<()> {
        sqlx::query("UPDATE users SET password_hash = ?1, updated_at = ?2 WHERE id = ?3")
            .bind(password_hash)
            .bind(Utc::now().with_timezone(&Shanghai).to_rfc3339())
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;
        Ok(())
    }

    pub async fn update(&self, id: &str, input: UpdateUser) -> AppResult<User> {
        let mut user = self.find_by_id(id).await?;

        if let Some(username) = input.username {
            user.username = username;
        }
        if let Some(is_admin) = input.is_admin {
            user.is_admin = is_admin;
        }

        sqlx::query("UPDATE users SET username = ?1, is_admin = ?2, updated_at = ?3 WHERE id = ?4")
            .bind(&user.username)
            .bind(user.is_admin as i32)
            .bind(Utc::now().with_timezone(&Shanghai).to_rfc3339())
            .bind(&user.id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                if e.to_string().contains("UNIQUE constraint failed") {
                    AppError::DuplicateUsername(user.username.clone())
                } else {
                    AppError::Database(e)
                }
            })?;

        Ok(user)
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM users WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }
}
