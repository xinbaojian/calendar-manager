# CalendarSync Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 构建一个轻量级日程管理服务，支持多用户、重复日程、iPhone 日历订阅、Webhook 通知和 Web 管理界面。

**Architecture:** 单体模块化架构，使用 Axum 构建 REST API，Askama 实现服务端渲染，SQLite 持久化数据，Tokio 处理异步任务。

**Tech Stack:** Rust 1.75+, Axum 0.7, SQLx, SQLite, Askama, Tokio, Reqwest, Docker

---

## Phase 1: 项目初始化和基础设施

### Task 1.1: 创建项目结构和 Cargo 配置

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `config.toml`
- Create: `.gitignore`
- Create: `Dockerfile`
- Create: `docker-compose.yml`

**Step 1: 创建 Cargo.toml**

```toml
[package]
name = "calendarsync"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["fs", "cors"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite", "chrono", "uuid"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
toml = "0.8"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
askama = "0.12"
reqwest = { version = "0.11", features = ["json"] }
hmac = "0.12"
sha2 = "0.10"
hex = "0.4"
anyhow = "1"
thiserror = "1"

[dev-dependencies]
http-body-util = "0.1"
tower = { version = "0.4", features = ["util"] }
```

**Step 2: 创建基础 main.rs**

```rust
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("CalendarSync starting...");
    Ok(())
}
```

**Step 3: 创建配置文件 config.toml**

```toml
[server]
host = "127.0.0.1"
port = 8080

[database]
path = "./data/calendar.db"

[auth]
admin_api_key = "admin-secret-key-change-me"

[cleanup]
check_interval_hours = 1
auto_delete_expired_days = 30

[webhook]
timeout_seconds = 10
max_retries = 3
```

**Step 4: 创建 .gitignore**

```
/target
/data
*.db
.env
config.local.toml
.DS_Store
```

**Step 5: 创建 Dockerfile**

```dockerfile
FROM rust:1.75-alpine AS builder
WORKDIR /app
RUN apk add --no-cache musl-dev
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM alpine:3.19
RUN apk add --no-cache ca-certificates tzdata
COPY --from=builder /app/target/release/calendarsync /usr/local/bin/
EXPOSE 8080
CMD ["calendarsync"]
```

**Step 6: 创建 docker-compose.yml**

```yaml
version: '3.8'
services:
  calendarsync:
    build: .
    container_name: calendarsync
    restart: unless-stopped
    ports:
      - "8080:8080"
    volumes:
      - ./data:/app/data
      - ./config.toml:/app/config.toml:ro
    environment:
      - TZ=Asia/Shanghai
    mem_limit: 100m
```

**Step 7: 验证编译**

Run: `cargo check`
Expected: 编译成功，无错误

**Step 8: 提交**

```bash
git add .
git commit -m "feat: initialize project structure and dependencies"
```

---

### Task 1.2: 配置加载模块

**Files:**
- Create: `src/config.rs`
- Modify: `src/main.rs`

**Step 1: 创建配置结构体**

```rust
// src/config.rs
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub cleanup: CleanupConfig,
    pub webhook: WebhookConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthConfig {
    pub admin_api_key: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CleanupConfig {
    #[serde(default = "default_check_interval")]
    pub check_interval_hours: u64,
    #[serde(default = "default_auto_delete_days")]
    pub auto_delete_expired_days: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebhookConfig {
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_check_interval() -> u64 { 1 }
fn default_auto_delete_days() -> u64 { 30 }
fn default_timeout() -> u64 { 10 }
fn default_max_retries() -> u32 { 3 }

pub fn load_config(path: &Path) -> anyhow::Result<Config> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}
```

**Step 2: 更新 main.rs 加载配置**

```rust
use calendarsync::config::load_config;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config(Path::new("config.toml"))?;
    println!("CalendarSync starting on {}:{}", config.server.host, config.server.port);
    Ok(())
}
```

**Step 3: 在 src/lib.rs 暴露模块**

```rust
pub mod config;
```

**Step 4: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 5: 提交**

```bash
git add src/
git commit -m "feat: add configuration loading module"
```

---

### Task 1.3: 错误处理模块

**Files:**
- Create: `src/error.rs`
- Modify: `src/lib.rs`

**Step 1: 创建错误类型**

```rust
// src/error.rs
use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Event not found: {0}")]
    EventNotFound(String),

    #[error("Webhook not found: {0}")]
    WebhookNotFound(String),

    #[error("Invalid API key")]
    InvalidApiKey,

    #[error("Insufficient permissions")]
    InsufficientPermissions,

    #[error("Duplicate username: {0}")]
    DuplicateUsername(String),

    #[error("Invalid recurrence rule: {0}")]
    InvalidRecurrenceRule(String),

    #[error("Webhook delivery failed: {0}")]
    WebhookDeliveryFailed(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "DATABASE_ERROR", self.to_string()),
            AppError::UserNotFound(_) => (StatusCode::NOT_FOUND, "USER_NOT_FOUND", self.to_string()),
            AppError::EventNotFound(_) => (StatusCode::NOT_FOUND, "EVENT_NOT_FOUND", self.to_string()),
            AppError::WebhookNotFound(_) => (StatusCode::NOT_FOUND, "WEBHOOK_NOT_FOUND", self.to_string()),
            AppError::InvalidApiKey => (StatusCode::UNAUTHORIZED, "INVALID_API_KEY", "API Key 无效".to_string()),
            AppError::InsufficientPermissions => (StatusCode::FORBIDDEN, "INSUFFICIENT_PERMISSIONS", "权限不足".to_string()),
            AppError::DuplicateUsername(_) => (StatusCode::CONFLICT, "DUPLICATE_USERNAME", self.to_string()),
            AppError::InvalidRecurrenceRule(_) => (StatusCode::BAD_REQUEST, "INVALID_RECURRENCE_RULE", self.to_string()),
            AppError::WebhookDeliveryFailed(_) => (StatusCode::INTERNAL_SERVER_ERROR, "WEBHOOK_DELIVERY_FAILED", self.to_string()),
            AppError::Serialization(_) => (StatusCode::BAD_REQUEST, "SERIALIZATION_ERROR", self.to_string()),
            AppError::Io(_) => (StatusCode::INTERNAL_SERVER_ERROR, "IO_ERROR", self.to_string()),
        };

        let body = json!({
            "error": {
                "code": code,
                "message": message,
                "details": {}
            }
        });

        (status, Json(body)).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;
```

**Step 2: 在 lib.rs 中暴露**

```rust
pub mod config;
pub mod error;
pub use error::{AppError, AppResult};
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 4: 提交**

```bash
git add src/
git commit -m "feat: add error handling module with HTTP error responses"
```

---

## Phase 2: 数据库层

### Task 2.1: 数据库迁移脚本

**Files:**
- Create: `migrations/001_initial_schema.sql`

**Step 1: 创建数据库 Schema**

```sql
-- migrations/001_initial_schema.sql

-- 用户表
CREATE TABLE users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    api_key TEXT NOT NULL UNIQUE,
    is_admin INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- 日程表
CREATE TABLE events (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    location TEXT,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    recurrence_rule TEXT,
    recurrence_until TEXT,
    reminder_minutes INTEGER,
    tags TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_events_user_time ON events(user_id, start_time);
CREATE INDEX idx_events_status ON events(status);
CREATE INDEX idx_events_user_status ON events(user_id, status);

-- Webhook 表
CREATE TABLE webhooks (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    url TEXT NOT NULL,
    events TEXT NOT NULL,
    secret TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_webhooks_user ON webhooks(user_id);

-- Webhook 日志表
CREATE TABLE webhook_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    webhook_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload TEXT,
    status_code INTEGER,
    response_body TEXT,
    sent_at TEXT NOT NULL
);

CREATE INDEX idx_webhook_logs_webhook ON webhook_logs(webhook_id);
```

**Step 2: 提交**

```bash
git add migrations/
git commit -m "feat: add initial database schema migration"
```

---

### Task 2.2: 数据库连接池

**Files:**
- Create: `src/db/mod.rs`
- Create: `src/db/pool.rs`

**Step 1: 创建连接池模块**

```rust
// src/db/mod.rs
pub mod pool;

pub use pool::create_pool;
```

```rust
// src/db/pool.rs
use sqlx::{Pool, Sqlite, SqlitePool};
use std::str::FromStr;

pub async fn create_pool(database_url: &str) -> anyhow::Result<SqlitePool> {
    // 确保数据目录存在
    if let Some(parent) = std::path::Path::new(database_url).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let pool = SqlitePool::connect(database_url).await?;

    // 启用外键约束
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &Pool<Sqlite>) -> anyhow::Result<()> {
    let migration_sql = std::fs::read_to_string("migrations/001_initial_schema.sql")?;
    let mut tx = pool.begin().await?;

    for statement in migration_sql.split(';') {
        let statement = statement.trim();
        if !statement.is_empty() {
            sqlx::query(statement).execute(&mut *tx).await?;
        }
    }

    tx.commit().await?;
    Ok(())
}
```

**Step 2: 在 lib.rs 中暴露**

```rust
pub mod config;
pub mod db;
pub mod error;
pub use error::{AppError, AppResult};
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 4: 提交**

```bash
git add src/
git commit -m "feat: add database connection pool and migrations"
```

---

### Task 2.3: 数据模型

**Files:**
- Create: `src/models/mod.rs`
- Create: `src/models/user.rs`
- Create: `src/models/event.rs`
- Create: `src/models/webhook.rs`

**Step 1: 创建用户模型**

```rust
// src/models/user.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub username: String,
    pub api_key: String,
    pub is_admin: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub is_admin: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub username: Option<String>,
}

impl User {
    pub fn new(username: String, is_admin: bool) -> Self {
        let id = format!("usr_{}", Uuid::new_v4());
        let api_key = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        Self {
            id,
            username,
            api_key,
            is_admin,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}
```

**Step 2: 创建日程模型**

```rust
// src/models/event.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Event {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub recurrence_rule: Option<String>,
    pub recurrence_until: Option<String>,
    pub reminder_minutes: Option<i32>,
    pub tags: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateEvent {
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub recurrence_rule: Option<String>,
    pub recurrence_until: Option<String>,
    pub reminder_minutes: Option<i32>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateEvent {
    pub title: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub recurrence_rule: Option<String>,
    pub recurrence_until: Option<String>,
    pub reminder_minutes: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct QueryEvents {
    pub user_id: Option<String>,
    pub status: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub keyword: Option<String>,
}

impl Event {
    pub fn new(user_id: String, input: CreateEvent) -> Self {
        let id = format!("evt_{}", Uuid::new_v4());
        let now = Utc::now().to_rfc3339();
        let tags = input.tags.map(|t| serde_json::to_string(&t).unwrap());

        Self {
            id,
            user_id,
            title: input.title,
            description: input.description,
            location: input.location,
            start_time: input.start_time,
            end_time: input.end_time,
            recurrence_rule: input.recurrence_rule,
            recurrence_until: input.recurrence_until,
            reminder_minutes: input.reminder_minutes,
            tags,
            status: "active".to_string(),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}
```

**Step 3: 创建 Webhook 模型**

```rust
// src/models/webhook.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Webhook {
    pub id: String,
    pub user_id: String,
    pub url: String,
    pub events: String,
    pub secret: Option<String>,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateWebhook {
    pub url: String,
    pub events: Vec<String>,
    pub secret: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWebhook {
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct WebhookLog {
    pub id: i64,
    pub webhook_id: String,
    pub event_type: String,
    pub payload: Option<String>,
    pub status_code: Option<i32>,
    pub response_body: Option<String>,
    pub sent_at: String,
}

impl Webhook {
    pub fn new(user_id: String, input: CreateWebhook) -> Self {
        let id = format!("wh_{}", Uuid::new_v4());
        let events = serde_json::to_string(&input.events).unwrap();
        let now = Utc::now().to_rfc3339();

        Self {
            id,
            user_id,
            url: input.url,
            events,
            secret: input.secret,
            is_active: true,
            created_at: now,
        }
    }
}
```

**Step 4: 创建模块导出**

```rust
// src/models/mod.rs
pub mod user;
pub mod event;
pub mod webhook;

pub use user::{User, CreateUser, UpdateUser};
pub use event::{Event, CreateEvent, UpdateEvent, QueryEvents};
pub use webhook::{Webhook, CreateWebhook, UpdateWebhook, WebhookPayload, WebhookLog};
```

**Step 5: 在 lib.rs 中暴露**

```rust
pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub use error::{AppError, AppResult};
```

**Step 6: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 7: 提交**

```bash
git add src/
git commit -m "feat: add data models for User, Event, and Webhook"
```

---

### Task 2.4: Repository 层 - User

**Files:**
- Create: `src/db/repositories/mod.rs`
- Create: `src/db/repositories/user.rs`

**Step 1: 创建 User Repository**

```rust
// src/db/repositories/user.rs
use crate::models::{User, CreateUser, UpdateUser};
use crate::error::{AppError, AppResult};
use sqlx::{Pool, Sqlite};

pub struct UserRepository {
    pool: Pool<Sqlite>,
}

impl UserRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, input: CreateUser) -> AppResult<User> {
        let user = User::new(input.username, input.is_admin.unwrap_or(false));

        sqlx::query(
            "INSERT INTO users (id, username, api_key, is_admin, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        )
        .bind(&user.id)
        .bind(&user.username)
        .bind(&user.api_key)
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
            .map_err(|_| AppError::UserNotFound(username.to_string()))
    }

    pub async fn list(&self) -> AppResult<Vec<User>> {
        sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn update(&self, id: &str, input: UpdateUser) -> AppResult<User> {
        let mut user = self.find_by_id(id).await?;

        if let Some(username) = input.username {
            user.username = username;
        }

        sqlx::query("UPDATE users SET username = ?1, updated_at = ?2 WHERE id = ?3")
            .bind(&user.username)
            .bind(&Utc::now().to_rfc3339())
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
```

**Step 2: 验证编译**

Run: `cargo check`
Expected: 可能缺少 Utc import，在 user.rs 顶部添加 `use chrono::Utc;`

**Step 3: 创建 repositories mod**

```rust
// src/db/repositories/mod.rs
pub mod user;
pub use user::UserRepository;
```

**Step 4: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 5: 提交**

```bash
git add src/
git commit -m "feat: add UserRepository with CRUD operations"
```

---

### Task 2.5: Repository 层 - Event

**Files:**
- Create: `src/db/repositories/event.rs`

**Step 1: 创建 Event Repository**

```rust
// src/db/repositories/event.rs
use crate::models::{Event, CreateEvent, UpdateEvent, QueryEvents};
use crate::error::{AppError, AppResult};
use sqlx::{Pool, Sqlite};

pub struct EventRepository {
    pool: Pool<Sqlite>,
}

impl EventRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, user_id: String, input: CreateEvent) -> AppResult<Event> {
        let event = Event::new(user_id, input);

        sqlx::query(
            "INSERT INTO events (id, user_id, title, description, location, start_time, end_time,
             recurrence_rule, recurrence_until, reminder_minutes, tags, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)"
        )
        .bind(&event.id)
        .bind(&event.user_id)
        .bind(&event.title)
        .bind(&event.description)
        .bind(&event.location)
        .bind(&event.start_time)
        .bind(&event.end_time)
        .bind(&event.recurrence_rule)
        .bind(&event.recurrence_until)
        .bind(event.reminder_minutes)
        .bind(&event.tags)
        .bind(&event.status)
        .bind(&event.created_at)
        .bind(&event.updated_at)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(event)
    }

    pub async fn find_by_id(&self, id: &str) -> AppResult<Event> {
        sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = ?1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| AppError::EventNotFound(id.to_string()))
    }

    pub async fn find_by_user(&self, user_id: &str, query: QueryEvents) -> AppResult<Vec<Event>> {
        let mut sql = "SELECT * FROM events WHERE user_id = ?1".to_string();
        let mut params = vec![user_id.to_string()];

        if let Some(status) = &query.status {
            sql.push_str(&format!(" AND status = ?{}", params.len() + 1));
            params.push(status.clone());
        }

        if let Some(from) = &query.from {
            sql.push_str(&format!(" AND start_time >= ?{}", params.len() + 1));
            params.push(from.clone());
        }

        if let Some(to) = &query.to {
            sql.push_str(&format!(" AND start_time <= ?{}", params.len() + 1));
            params.push(to.clone());
        }

        if let Some(keyword) = &query.keyword {
            sql.push_str(&format!(" AND (title LIKE ?{} OR description LIKE ?{})", params.len() + 1, params.len() + 2));
            let pattern = format!("%{}%", keyword);
            params.push(pattern.clone());
            params.push(pattern);
        }

        sql.push_str(" ORDER BY start_time ASC");

        let mut query = sqlx::query_as::<_, Event>(&sql);
        for param in params {
            query = query.bind(param);
        }

        query.fetch_all(&self.pool).await.map_err(AppError::Database)
    }

    pub async fn update(&self, id: &str, input: UpdateEvent) -> AppResult<Event> {
        let mut event = self.find_by_id(id).await?;
        let now = Utc::now().to_rfc3339();

        if let Some(title) = input.title {
            event.title = title;
        }
        if let Some(description) = input.description {
            event.description = Some(description);
        }
        if let Some(location) = input.location {
            event.location = Some(location);
        }
        if let Some(start_time) = input.start_time {
            event.start_time = start_time;
        }
        if let Some(end_time) = input.end_time {
            event.end_time = end_time;
        }
        if let Some(recurrence_rule) = input.recurrence_rule {
            event.recurrence_rule = Some(recurrence_rule);
        }
        if let Some(recurrence_until) = input.recurrence_until {
            event.recurrence_until = Some(recurrence_until);
        }
        if let Some(reminder_minutes) = input.reminder_minutes {
            event.reminder_minutes = Some(reminder_minutes);
        }
        if let Some(tags) = input.tags {
            event.tags = Some(serde_json::to_string(&tags).unwrap());
        }
        if let Some(status) = input.status {
            event.status = status;
        }

        sqlx::query(
            "UPDATE events SET title = ?1, description = ?2, location = ?3, start_time = ?4,
             end_time = ?5, recurrence_rule = ?6, recurrence_until = ?7, reminder_minutes = ?8,
             tags = ?9, status = ?10, updated_at = ?11 WHERE id = ?12"
        )
        .bind(&event.title)
        .bind(&event.description)
        .bind(&event.location)
        .bind(&event.start_time)
        .bind(&event.end_time)
        .bind(&event.recurrence_rule)
        .bind(&event.recurrence_until)
        .bind(event.reminder_minutes)
        .bind(&event.tags)
        .bind(&event.status)
        .bind(&now)
        .bind(&event.id)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(event)
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM events WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }

    pub async fn mark_expired(&self, before: &str) -> AppResult<u64> {
        let result = sqlx::query(
            "UPDATE events SET status = 'expired', updated_at = ?1
             WHERE status = 'active' AND end_time < ?2"
        )
        .bind(&Utc::now().to_rfc3339())
        .bind(before)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(result.rows_affected())
    }

    pub async fn delete_old_expired(&self, days: i64) -> AppResult<u64> {
        let cutoff = Utc::now() - chrono::Duration::days(days);

        let result = sqlx::query("DELETE FROM events WHERE status = 'expired' AND updated_at < ?1")
            .bind(cutoff.to_rfc3339())
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;

        Ok(result.rows_affected())
    }
}
```

**Step 2: 更新 repositories mod**

```rust
// src/db/repositories/mod.rs
pub mod user;
pub mod event;
pub use user::UserRepository;
pub use event::EventRepository;
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 4: 提交**

```bash
git add src/
git commit -m "feat: add EventRepository with CRUD and expiration management"
```

---

### Task 2.6: Repository 层 - Webhook

**Files:**
- Create: `src/db/repositories/webhook.rs`

**Step 1: 创建 Webhook Repository**

```rust
// src/db/repositories/webhook.rs
use crate::models::{Webhook, CreateWebhook, UpdateWebhook, WebhookLog};
use crate::error::{AppError, AppResult};
use sqlx::{Pool, Sqlite};

pub struct WebhookRepository {
    pool: Pool<Sqlite>,
}

impl WebhookRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, user_id: String, input: CreateWebhook) -> AppResult<Webhook> {
        let webhook = Webhook::new(user_id, input);

        sqlx::query(
            "INSERT INTO webhooks (id, user_id, url, events, secret, is_active, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
        )
        .bind(&webhook.id)
        .bind(&webhook.user_id)
        .bind(&webhook.url)
        .bind(&webhook.events)
        .bind(&webhook.secret)
        .bind(webhook.is_active as i32)
        .bind(&webhook.created_at)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(webhook)
    }

    pub async fn find_by_id(&self, id: &str) -> AppResult<Webhook> {
        sqlx::query_as::<_, Webhook>("SELECT * FROM webhooks WHERE id = ?1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| AppError::WebhookNotFound(id.to_string()))
    }

    pub async fn find_by_user(&self, user_id: &str) -> AppResult<Vec<Webhook>> {
        sqlx::query_as::<_, Webhook>("SELECT * FROM webhooks WHERE user_id = ?1 ORDER BY created_at DESC")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn find_active_by_user(&self, user_id: &str) -> AppResult<Vec<Webhook>> {
        sqlx::query_as::<_, Webhook>("SELECT * FROM webhooks WHERE user_id = ?1 AND is_active = 1 ORDER BY created_at DESC")
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn update(&self, id: &str, input: UpdateWebhook) -> AppResult<Webhook> {
        let mut webhook = self.find_by_id(id).await?;

        if let Some(url) = input.url {
            webhook.url = url;
        }
        if let Some(events) = input.events {
            webhook.events = serde_json::to_string(&events).unwrap();
        }
        if let Some(is_active) = input.is_active {
            webhook.is_active = is_active;
        }

        sqlx::query("UPDATE webhooks SET url = ?1, events = ?2, is_active = ?3 WHERE id = ?4")
            .bind(&webhook.url)
            .bind(&webhook.events)
            .bind(webhook.is_active as i32)
            .bind(&webhook.id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;

        Ok(webhook)
    }

    pub async fn delete(&self, id: &str) -> AppResult<()> {
        sqlx::query("DELETE FROM webhooks WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }

    pub async fn log_delivery(
        &self,
        webhook_id: &str,
        event_type: &str,
        payload: &str,
        status_code: Option<i32>,
        response_body: Option<String>,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO webhook_logs (webhook_id, event_type, payload, status_code, response_body, sent_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        )
        .bind(webhook_id)
        .bind(event_type)
        .bind(payload)
        .bind(status_code)
        .bind(response_body)
        .bind(&Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }
}
```

**Step 2: 更新 repositories mod**

```rust
// src/db/repositories/mod.rs
pub mod user;
pub mod event;
pub mod webhook;
pub use user::UserRepository;
pub use event::EventRepository;
pub use webhook::WebhookRepository;
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 4: 提交**

```bash
git add src/
git commit -m "feat: add WebhookRepository with logging"
```

---

## Phase 3: 业务逻辑层

### Task 3.1: 认证中间件

**Files:**
- Create: `src/handlers/mod.rs`
- Create: `src/handlers/auth.rs`

**Step 1: 创建认证中间件**

```rust
// src/handlers/auth.rs
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

pub struct AuthenticatedUser {
    pub user: User,
    pub is_admin: bool,
}

pub async fn auth_middleware(
    State(user_repo): State<UserRepository>,
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
```

**Step 2: 创建 handlers mod**

```rust
// src/handlers/mod.rs
pub mod auth;
pub use auth::{AuthenticatedUser, require_admin, check_user_access, auth_middleware};
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 4: 提交**

```bash
git add src/
git commit -m "feat: add authentication middleware"
```

---

### Task 3.2: User Handlers

**Files:**
- Create: `src/handlers/users.rs`

**Step 1: 创建用户 API handlers**

```rust
// src/handlers/users.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use crate::{
    db::repositories::UserRepository,
    handlers::{AuthenticatedUser, require_admin},
    models::{CreateUser, UpdateUser, User},
    error::AppResult,
};

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
```

**Step 2: 更新 handlers mod**

```rust
// src/handlers/mod.rs
pub mod auth;
pub mod users;
pub use auth::{AuthenticatedUser, require_admin, check_user_access, auth_middleware};
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 4: 提交**

```bash
git add src/
git commit -m "feat: add user API handlers"
```

---

### Task 3.3: Event Handlers

**Files:**
- Create: `src/handlers/events.rs`

**Step 1: 创建事件 API handlers**

```rust
// src/handlers/events.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use crate::{
    db::repositories::{EventRepository, UserRepository},
    handlers::{AuthenticatedUser, check_user_access},
    models::{CreateEvent, UpdateEvent, QueryEvents, Event},
    error::AppResult,
};

#[derive(Serialize)]
pub struct EventResponse {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: String,
    pub end_time: String,
    pub recurrence_rule: Option<String>,
    pub recurrence_until: Option<String>,
    pub reminder_minutes: Option<i32>,
    pub tags: Option<Vec<String>>,
    pub status: String,
    pub created_at: String,
}

impl TryFrom<Event> for EventResponse {
    type Error = serde_json::Error;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        Ok(Self {
            id: event.id,
            user_id: event.user_id,
            title: event.title,
            description: event.description,
            location: event.location,
            start_time: event.start_time,
            end_time: event.end_time,
            recurrence_rule: event.recurrence_rule,
            recurrence_until: event.recurrence_until,
            reminder_minutes: event.reminder_minutes,
            tags: event.tags.as_ref().and_then(|t| serde_json::from_str(t).ok()),
            status: event.status,
            created_at: event.created_at,
        })
    }
}

#[derive(Deserialize)]
pub struct CreateEventRequest {
    pub user_id: String,
    pub event: CreateEvent,
}

#[derive(Deserialize)]
pub struct EventQuery {
    pub user_id: Option<String>,
    pub status: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub keyword: Option<String>,
}

pub async fn create_event(
    State(event_repo): State<EventRepository>,
    State(user_repo): State<UserRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
    Json(req): Json<CreateEventRequest>,
) -> AppResult<Json<EventResponse>> {
    check_user_access(&auth, &req.user_id)?;

    let event = event_repo.create(req.user_id, req.event).await?;
    Ok(Json(EventResponse::try_from(event)?))
}

pub async fn list_events(
    State(event_repo): State<EventRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
    Query(query): Query<EventQuery>,
) -> AppResult<Json<Vec<EventResponse>>> {
    let user_id = query.user_id.clone().unwrap_or_else(|| auth.user.id.clone());

    check_user_access(&auth, &user_id)?;

    let query = QueryEvents {
        user_id: Some(user_id),
        status: query.status,
        from: query.from,
        to: query.to,
        keyword: query.keyword,
    };

    let events = event_repo.find_by_user(&query.user_id.unwrap(), query).await?;
    let response: Vec<EventResponse> = events
        .into_iter()
        .filter_map(|e| EventResponse::try_from(e).ok())
        .collect();

    Ok(Json(response))
}

pub async fn get_event(
    State(event_repo): State<EventRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> AppResult<Json<EventResponse>> {
    let event = event_repo.find_by_id(&id).await?;
    check_user_access(&auth, &event.user_id)?;

    Ok(Json(EventResponse::try_from(event)?))
}

pub async fn update_event(
    State(event_repo): State<EventRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
    Json(input): Json<UpdateEvent>,
) -> AppResult<Json<EventResponse>> {
    let event = event_repo.find_by_id(&id).await?;
    check_user_access(&auth, &event.user_id)?;

    let event = event_repo.update(&id, input).await?;
    Ok(Json(EventResponse::try_from(event)?))
}

pub async fn delete_event(
    State(event_repo): State<EventRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let event = event_repo.find_by_id(&id).await?;
    check_user_access(&auth, &event.user_id)?;

    event_repo.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn search_events(
    State(event_repo): State<EventRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
    Query(query): Query<EventQuery>,
) -> AppResult<Json<Vec<EventResponse>>> {
    let user_id = query.user_id.clone().unwrap_or_else(|| auth.user.id.clone());

    check_user_access(&auth, &user_id)?;

    let query = QueryEvents {
        user_id: Some(user_id),
        status: query.status.or(Some("all".to_string())),
        from: query.from,
        to: query.to,
        keyword: query.keyword,
    };

    let events = event_repo.find_by_user(&query.user_id.unwrap(), query).await?;
    let response: Vec<EventResponse> = events
        .into_iter()
        .filter_map(|e| EventResponse::try_from(e).ok())
        .collect();

    Ok(Json(response))
}
```

**Step 2: 更新 handlers mod**

```rust
// src/handlers/mod.rs
pub mod auth;
pub mod users;
pub mod events;
pub use auth::{AuthenticatedUser, require_admin, check_user_access, auth_middleware};
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 4: 提交**

```bash
git add src/
git commit -m "feat: add event API handlers"
```

---

### Task 3.4: Webhook Handlers

**Files:**
- Create: `src/handlers/webhooks.rs`

**Step 1: 创建 Webhook API handlers**

```rust
// src/handlers/webhooks.rs
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use crate::{
    db::repositories::{WebhookRepository, UserRepository},
    handlers::{AuthenticatedUser, check_user_access},
    models::{CreateWebhook, UpdateWebhook, Webhook},
    error::AppResult,
};

#[derive(Serialize)]
pub struct WebhookResponse {
    pub id: String,
    pub user_id: String,
    pub url: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub created_at: String,
}

impl TryFrom<Webhook> for WebhookResponse {
    type Error = serde_json::Error;

    fn try_from(webhook: Webhook) -> Result<Self, Self::Error> {
        Ok(Self {
            id: webhook.id,
            user_id: webhook.user_id,
            url: webhook.url,
            events: serde_json::from_str(&webhook.events)?,
            is_active: webhook.is_active,
            created_at: webhook.created_at,
        })
    }
}

#[derive(Deserialize)]
pub struct CreateWebhookRequest {
    pub user_id: String,
    pub webhook: CreateWebhook,
}

pub async fn create_webhook(
    State(webhook_repo): State<WebhookRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
    Json(req): Json<CreateWebhookRequest>,
) -> AppResult<Json<WebhookResponse>> {
    check_user_access(&auth, &req.user_id)?;

    let webhook = webhook_repo.create(req.user_id, req.webhook).await?;
    Ok(Json(WebhookResponse::try_from(webhook)?))
}

pub async fn list_webhooks(
    State(webhook_repo): State<WebhookRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
) -> AppResult<Json<Vec<WebhookResponse>>> {
    let webhooks = webhook_repo.find_by_user(&auth.user.id).await?;
    let response: Vec<WebhookResponse> = webhooks
        .into_iter()
        .filter_map(|w| WebhookResponse::try_from(w).ok())
        .collect();

    Ok(Json(response))
}

pub async fn get_webhook(
    State(webhook_repo): State<WebhookRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> AppResult<Json<WebhookResponse>> {
    let webhook = webhook_repo.find_by_id(&id).await?;
    check_user_access(&auth, &webhook.user_id)?;

    Ok(Json(WebhookResponse::try_from(webhook)?))
}

pub async fn update_webhook(
    State(webhook_repo): State<WebhookRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
    Json(input): Json<UpdateWebhook>,
) -> AppResult<Json<WebhookResponse>> {
    let webhook = webhook_repo.find_by_id(&id).await?;
    check_user_access(&auth, &webhook.user_id)?;

    let webhook = webhook_repo.update(&id, input).await?;
    Ok(Json(WebhookResponse::try_from(webhook)?))
}

pub async fn delete_webhook(
    State(webhook_repo): State<WebhookRepository>,
    Extension(auth): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> AppResult<StatusCode> {
    let webhook = webhook_repo.find_by_id(&id).await?;
    check_user_access(&auth, &webhook.user_id)?;

    webhook_repo.delete(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}
```

**Step 2: 更新 handlers mod**

```rust
// src/handlers/mod.rs
pub mod auth;
pub mod users;
pub mod events;
pub mod webhooks;
pub use auth::{AuthenticatedUser, require_admin, check_user_access, auth_middleware};
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 4: 提交**

```bash
git add src/
git commit -m "feat: add webhook API handlers"
```

---

## Phase 4: iCalendar 服务

### Task 4.1: iCalendar 生成器

**Files:**
- Create: `src/ical/mod.rs`
- Create: `src/ical/generator.rs`

**Step 1: 创建 iCalendar 生成器**

```rust
// src/ical/generator.rs
use crate::models::Event;
use chrono::{DateTime, Utc};

pub struct ICalGenerator;

impl ICalGenerator {
    pub fn generate(events: &[Event], calendar_name: &str) -> String {
        let mut ical = String::new();

        ical.push_str("BEGIN:VCALENDAR\r\n");
        ical.push_str("VERSION:2.0\r\n");
        ical.push_str("PRODID:-//CalendarSync//CN\r\n");
        ical.push_str("CALSCALE:GREGORIAN\r\n");
        ical.push_str("METHOD:PUBLISH\r\n");
        ical.push_str(&format!("X-WR-CALNAME:{}\r\n", calendar_name));
        ical.push_str("X-WR-CALDESC:CalendarSync 日程订阅\r\n");

        for event in events {
            if event.status != "active" {
                continue;
            }

            ical.push_str("BEGIN:VEVENT\r\n");

            // 格式化时间
            if let Ok(dt) = DateTime::parse_from_rfc3339(&event.start_time) {
                ical.push_str(&format!("DTSTART:{}\r\n", dt.format("%Y%m%dT%H%M%SZ")));
            }
            if let Ok(dt) = DateTime::parse_from_rfc3339(&event.end_time) {
                ical.push_str(&format!("DTEND:{}\r\n", dt.format("%Y%m%dT%H%M%SZ")));
            }

            ical.push_str(&format!("DTSTAMP:{}\r\n", Utc::now().format("%Y%m%dT%H%M%SZ")));
            ical.push_str(&format!("UID:{}@calendarsync\r\n", event.id));
            ical.push_str(&format!("SUMMARY:{}\r\n", escape_ical_text(&event.title)));

            if let Some(desc) = &event.description {
                ical.push_str(&format!("DESCRIPTION:{}\r\n", escape_ical_text(desc)));
            }

            if let Some(location) = &event.location {
                ical.push_str(&format!("LOCATION:{}\r\n", escape_ical_text(location)));
            }

            ical.push_str("STATUS:CONFIRMED\r\n");

            if let Some(minutes) = event.reminder_minutes {
                ical.push_str("BEGIN:VALARM\r\n");
                ical.push_str(&format!("TRIGGER:-PT{}M\r\n", minutes));
                ical.push_str("ACTION:DISPLAY\r\n");
                ical.push_str("DESCRIPTION:日程提醒\r\n");
                ical.push_str("END:VALARM\r\n");
            }

            ical.push_str("END:VEVENT\r\n");
        }

        ical.push_str("END:VCALENDAR\r\n");
        ical
    }
}

fn escape_ical_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('\n', "\\n")
}
```

**Step 2: 创建模块**

```rust
// src/ical/mod.rs
pub mod generator;
pub use generator::ICalGenerator;
```

**Step 3: 在 lib.rs 中暴露**

```rust
pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod handlers;
pub mod ical;
pub use error::{AppError, AppResult};
```

**Step 4: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 5: 提交**

```bash
git add src/
git commit -m "feat: add iCalendar generator"
```

---

### Task 4.2: 日历订阅 Handler

**Files:**
- Create: `src/handlers/calendar.rs`

**Step 1: 创建订阅接口 handler**

```rust
// src/handlers/calendar.rs
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::Response,
};
use crate::{
    db::repositories::{EventRepository, UserRepository},
    handlers::QueryEvents,
    ical::ICalGenerator,
    error::AppResult,
};

pub async fn subscribe_calendar(
    State(event_repo): State<EventRepository>,
    State(user_repo): State<UserRepository>,
    Path(user_id): Path<String>,
) -> AppResult<Response> {
    // 验证用户存在
    let user = user_repo.find_by_id(&user_id).await?;

    // 查询用户的有效日程
    let query = QueryEvents {
        user_id: Some(user_id),
        status: Some("active".to_string()),
        from: None,
        to: None,
        keyword: None,
    };

    let events = event_repo.find_by_user(&user.id, query).await?;

    // 生成 iCalendar
    let ical_content = ICalGenerator::generate(&events, &format!("{}的日程", user.username));

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/calendar; charset=utf-8")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(ical_content.into())
        .unwrap())
}
```

**Step 2: 更新 handlers mod**

```rust
// src/handlers/mod.rs
pub mod auth;
pub mod users;
pub mod events;
pub mod webhooks;
pub mod calendar;
pub use auth::{AuthenticatedUser, require_admin, check_user_access, auth_middleware};
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 4: 提交**

```bash
git add src/
git commit -m "feat: add calendar subscription endpoint"
```

---

## Phase 5: Web 界面

### Task 5.1: Askama 模板配置

**Files:**
- Create: `src/templates/mod.rs`
- Create: `src/templates/base.rs.html`
- Create: `src/templates/index.rs.html`

**Step 1: 创建基础模板**

```html
<!-- src/templates/base.rs.html -->
{% use crate::templates::Index %}

<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{% block title %}CalendarSync{% endblock %}</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; }
        .container { max-width: 1200px; margin: 0 auto; padding: 20px; }
        .header { background: #f5f5f5; padding: 20px 0; margin-bottom: 20px; }
        .nav { display: flex; gap: 20px; }
        .nav a { text-decoration: none; color: #333; }
        .nav a:hover { color: #007aff; }
    </style>
</head>
<body>
    <div class="header">
        <div class="container">
            <h1>CalendarSync</h1>
            <nav class="nav">
                <a href="/">首页</a>
                <a href="/events">日程</a>
                <a href="/settings">设置</a>
            </nav>
        </div>
    </div>
    <div class="container">
        {% block content %}{% endblock %}
    </div>
</body>
</html>
```

**Step 2: 创建首页模板**

```html
<!-- src/templates/index.rs.html -->
{% extends "base.rs.html" %}

{% block title %}CalendarSync - 首页{% endblock %}

{% block content %}
<h2>欢迎使用 CalendarSync</h2>
<p>轻量级日程管理服务</p>
{% endblock %}
```

**Step 3: 创建模块**

```rust
// src/templates/mod.rs
use askama::Template;

mod base;
mod index;

pub use index::IndexTemplate;
```

**Step 4: 在 main.rs 中添加模板**

```rust
use calendarsync::templates::IndexTemplate;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = load_config(Path::new("config.toml"))?;
    println!("CalendarSync starting on {}:{}", config.server.host, config.server.port);
    Ok(())
}
```

**Step 5: 验证编译**

Run: `cargo check`
Expected: Askama 会生成模板代码，编译成功

**Step 6: 提交**

```bash
git add src/
git commit -m "feat: add Askama template infrastructure"
```

---

## Phase 6: 主程序集成

### Task 6.1: HTTP 服务器设置

**Files:**
- Modify: `src/main.rs`

**Step 1: 完整的 main.rs**

```rust
use calendarsync::{
    config::{load_config, Config},
    db::{create_pool, run_migrations, repositories::{UserRepository, EventRepository, WebhookRepository}},
    handlers::{
        users::{create_user, list_users, get_user, update_user, delete_user},
        events::{create_event, list_events, get_event, update_event, delete_event, search_events},
        webhooks::{create_webhook, list_webhooks, get_webhook, update_webhook, delete_webhook},
        calendar::subscribe_calendar,
        auth_middleware,
    },
    templates::IndexTemplate,
};
use axum::{
    routing::{get, post, put, delete},
    Router,
    http::header::AUTHORIZATION,
    middleware,
};
use std::{path::Path, sync::Arc};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "calendarsync=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    let config = load_config(Path::new("config.toml"))?;

    // 创建数据库连接池
    let pool = create_pool(&format!("sqlite:{}", config.database.path)).await?;
    run_migrations(&pool).await?;

    // 创建 repositories
    let user_repo = Arc::new(UserRepository::new(pool.clone()));
    let event_repo = Arc::new(EventRepository::new(pool.clone()));
    let webhook_repo = Arc::new(WebhookRepository::new(pool));

    // 构建 Router
    let app = Router::new()
        // Web UI
        .route("/", get(index_handler))
        .route("/events", get(index_handler))
        .route("/settings", get(index_handler))

        // API Routes
        .route("/api/users", post(create_user).get(list_users))
        .route("/api/users/:id", get(get_user).put(update_user).delete(delete_user))

        .route("/api/events", post(create_event).get(list_events))
        .route("/api/events/search", get(search_events))
        .route("/api/events/:id", get(get_event).put(update_event).delete(delete_event))

        .route("/api/webhooks", post(create_webhook).get(list_webhooks))
        .route("/api/webhooks/:id", get(get_webhook).put(update_webhook).delete(delete_webhook))

        // Calendar subscription
        .route("/calendar/:user_id/subscribe.ics", get(subscribe_calendar))

        // 注入状态
        .with_state(user_repo)
        .with_state(event_repo)
        .with_state(webhook_repo)

        // 中间件
        .layer(middleware::from_fn_with_state(
            user_repo.clone(),
            auth_middleware,
        ))
        .layer(CorsLayer::permissive());

    // 启动服务器
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("CalendarSync listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn index_handler() -> IndexTemplate {
    IndexTemplate
}
```

**Step 2: 更新 lib.rs 暴露所有模块**

```rust
pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod handlers;
pub mod ical;
pub mod templates;

pub use error::{AppError, AppResult};
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 4: 提交**

```bash
git add src/
git commit -m "feat: integrate HTTP server with all routes"
```

---

## Phase 7: 后台任务和 Webhook 服务

### Task 7.1: Webhook 发送服务

**Files:**
- Create: `src/services/mod.rs`
- Create: `src/services/webhook.rs`

**Step 1: 创建 Webhook 服务**

```rust
// src/services/webhook.rs
use crate::{
    db::repositories::{WebhookRepository, EventRepository},
    models::WebhookPayload,
    error::{AppError, AppResult},
};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::time::Duration;
use tokio::time::sleep;

type HmacSha256 = Hmac<Sha256>;

pub struct WebhookService {
    webhook_repo: WebhookRepository,
    event_repo: EventRepository,
    timeout: Duration,
    max_retries: u32,
}

impl WebhookService {
    pub fn new(
        webhook_repo: WebhookRepository,
        event_repo: EventRepository,
        timeout_seconds: u64,
        max_retries: u32,
    ) -> Self {
        Self {
            webhook_repo,
            event_repo,
            timeout: Duration::from_secs(timeout_seconds),
            max_retries,
        }
    }

    pub async fn send_event_webhook(&self, user_id: &str, event_type: &str, data: serde_json::Value) -> AppResult<()> {
        let webhooks = self.webhook_repo.find_active_by_user(user_id).await?;

        for webhook in webhooks {
            let events: Vec<String> = serde_json::from_str(&webhook.events).unwrap();
            if !events.contains(&event_type.to_string()) {
                continue;
            }

            let payload = WebhookPayload {
                event_type: event_type.to_string(),
                data: data.clone(),
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            let payload_json = serde_json::to_string(&payload)?;

            if let Err(e) = self.send_with_retry(&webhook, &payload_json).await {
                tracing::error!("Webhook {} failed: {}", webhook.id, e);
            }
        }

        Ok(())
    }

    async fn send_with_retry(&self, webhook: &crate::models::Webhook, payload: &str) -> AppResult<()> {
        let mut last_error = None;

        for attempt in 1..=self.max_retries {
            let client = reqwest::Client::new();
            let mut request = client
                .post(&webhook.url)
                .header("Content-Type", "application/json")
                .body(payload.to_string())
                .timeout(self.timeout);

            // 添加签名
            if let Some(secret) = &webhook.secret {
                let signature = self.sign(payload, secret)?;
                request = request.header("X-Webhook-Signature", format!("sha256={}", signature));
            }

            match request.send().await {
                Ok(response) => {
                    let status = response.status().as_u16();
                    let body = response.text().await.unwrap_or_default();

                    self.webhook_repo.log_delivery(
                        &webhook.id,
                        "event",
                        payload,
                        Some(status as i32),
                        Some(body.clone()),
                    ).await?;

                    if status >= 200 && status < 300 {
                        return Ok(());
                    }

                    last_error = Some(AppError::WebhookDeliveryFailed(format!("Status: {}", status)));
                }
                Err(e) => {
                    last_error = Some(AppError::WebhookDeliveryFailed(e.to_string()));
                }
            }

            if attempt < self.max_retries {
                sleep(Duration::from_secs(2u64.pow(attempt - 1))).await;
            }
        }

        Err(last_error.unwrap())
    }

    fn sign(&self, payload: &str, secret: &str) -> AppResult<String> {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .map_err(|_| AppError::WebhookDeliveryFailed("Invalid secret".to_string()))?;
        mac.update(payload.as_bytes());
        Ok(hex::encode(mac.finalize().into_bytes()))
    }
}
```

**Step 2: 创建服务模块**

```rust
// src/services/mod.rs
pub mod webhook;
pub use webhook::WebhookService;
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 4: 提交**

```bash
git add src/
git commit -m "feat: add webhook delivery service with retry"
```

---

### Task 7.2: 定时清理任务

**Files:**
- Modify: `src/main.rs`

**Step 1: 在 main.rs 添加清理任务**

```rust
use calendarsync::{..., services::WebhookService};
use std::time::Duration;

// 在 main 函数中，启动服务器之前添加清理任务

// 启动定时清理任务
let event_repo_cleanup = event_repo.clone();
let webhook_repo_cleanup = webhook_repo.clone();
let cleanup_config = config.cleanup.clone();
let webhook_service = WebhookService::new(
    webhook_repo_cleanup.clone(),
    event_repo_cleanup.clone(),
    config.webhook.timeout_seconds,
    config.webhook.max_retries,
);

tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(cleanup_config.check_interval_hours * 3600));

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

        // 删除旧的过期事件
        if cleanup_config.auto_delete_expired_days > 0 {
            match event_repo_cleanup.delete_old_expired(cleanup_config.auto_delete_expired_days as i64).await {
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
```

**Step 2: 更新 lib.rs**

```rust
pub mod config;
pub mod db;
pub mod error;
pub mod models;
pub mod handlers;
pub mod ical;
pub mod templates;
pub mod services;

pub use error::{AppError, AppResult};
```

**Step 3: 验证编译**

Run: `cargo check`
Expected: 编译成功

**Step 4: 提交**

```bash
git add src/
git commit -m "feat: add scheduled cleanup task for expired events"
```

---

## Phase 8: 测试

### Task 8.1: 集成测试

**Files:**
- Create: `tests/integration/api_tests.rs`

**Step 1: 创建 API 集成测试**

```rust
// tests/integration/api_tests.rs
use axum::{
    body::Body,
    http::{header, Method, Request},
};
use http_body_util::BodyExt;
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_create_and_list_users() {
    // 测试用户创建和列表
}

#[tokio::test]
async fn test_create_and_list_events() {
    // 测试日程创建和列表
}

#[tokio::test]
async fn test_calendar_subscription() {
    // 测试日历订阅
}
```

**Step 2: 添加测试依赖到 Cargo.toml**

```toml
[dev-dependencies]
http-body-util = "0.1"
tower = { version = "0.4", features = ["util"] }
```

**Step 3: 验证测试编译**

Run: `cargo test`
Expected: 测试框架可以运行

**Step 4: 提交**

```bash
git add tests/
git commit -m "test: add integration test skeleton"
```

---

## Phase 9: 文档和部署

### Task 9.1: README 文档

**Files:**
- Create: `README.md`

**Step 1: 创建 README**

```markdown
# CalendarSync

轻量级日程管理服务，支持 iPhone 日历订阅。

## 功能特性

- 多用户支持
- RESTful API
- iPhone 日历订阅 (.ics)
- 重复日程
- Webhook 通知
- Web 管理界面

## 快速开始

### Docker 部署

```bash
docker-compose up -d
```

### 手动运行

```bash
cargo run
```

## API 文档

详见 [API 文档](docs/api.md)

## 配置

编辑 `config.toml` 文件。

## License

MIT
```

**Step 2: 提交**

```bash
git add README.md
git commit -m "docs: add README"
```

---

### Task 9.2: 最终验证

**Step 1: 完整构建**

Run: `cargo build --release`
Expected: 编译成功，生成二进制文件

**Step 2: 运行测试**

Run: `cargo test`
Expected: 所有测试通过

**Step 3: Docker 构建**

Run: `docker build -t calendarsync:latest .`
Expected: 镜像构建成功

**Step 4: 最终提交**

```bash
git add .
git commit -m "chore: final project structure verification"
```

---

## 总结

完成以上所有任务后，项目将包含：

1. ✅ 完整的 REST API
2. ✅ 多用户支持和认证
3. ✅ 日程 CRUD 和重复日程支持
4. ✅ iPhone 日历订阅 (.ics)
5. ✅ Webhook 通知系统
6. ✅ Web 管理界面
7. ✅ 定时清理任务
8. ✅ Docker 部署配置

项目可以部署到云服务器并通过 Nginx 反向代理暴露 HTTPS 服务。
