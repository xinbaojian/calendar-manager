# MCP HTTP 接口实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**目标:** 为 CalendarSync 添加 Model Context Protocol (MCP) HTTP 接口，允许其他 AI 工具通过标准协议调用日程管理功能。

**架构:** 使用官方 rmcp SDK 在现有 Axum 服务中集成 MCP 服务器，通过 `/mcp` 路由暴露日程管理工具，复用现有 API Key 认证机制和 Repository 层。

**技术栈:** Rust, Axum, rmcp (official MCP SDK), schemars (JSON Schema), SQLite

---

## Task 1: 添加 MCP 相关依赖

**文件:**
- 修改: `Cargo.toml`

**步骤 1: 添加依赖到 Cargo.toml**

```toml
[dependencies]
# 现有依赖保持不变...
tokio = { version = "1", features = ["full"] }
axum = "0.7"
# ... 其他依赖

# 新增 MCP 依赖
rmcp = { version = "0.16", features = ["server"] }
schemars = "0.8"

[dev-dependencies]
# 现有测试依赖...
# 新增 MCP 客户端用于测试
rmcp = { version = "0.16", features = ["server", "client"] }
```

**步骤 2: 验证依赖解析**

运行: `cargo check`
预期: 成功解析依赖，无编译错误

**步骤 3: 提交**

```bash
git add Cargo.toml
git commit -m "deps: 添加 rmcp 和 schemars 依赖"
```

---

## Task 2: 创建 MCP 模块结构

**文件:**
- 创建: `src/mcp/mod.rs`
- 创建: `src/mcp/models.rs`
- 创建: `src/mcp/transport.rs`
- 创建: `src/mcp/server.rs`
- 创建: `src/mcp/handlers.rs`

**步骤 1: 创建 MCP 模块入口**

编辑 `src/mcp/mod.rs`:

```rust
pub mod models;
pub mod transport;
pub mod server;
pub mod handlers;

pub use server::CalendarMCP;
pub use transport::create_mcp_router;
```

编辑 `src/main.rs`，在文件顶部添加:

```rust
mod mcp;
```

**步骤 2: 验证模块编译**

运行: `cargo check`
预期: 编译失败，因为引用的模块还不存在

**步骤 3: 创建占位文件以通过编译**

创建 `src/mcp/models.rs`:
```rust
// MCP 请求/响应模型 - Task 3 实现
```

创建 `src/mcp/transport.rs`:
```rust
// MCP HTTP 传输层 - Task 4 实现
```

创建 `src/mcp/server.rs`:
```rust
// MCP 服务器 - Task 5 实现
```

创建 `src/mcp/handlers.rs`:
```rust
// MCP 工具处理器 - Task 6 实现
```

**步骤 4: 再次验证编译**

运行: `cargo check`
预期: 编译通过，只有警告

**步骤 5: 提交**

```bash
git add src/mcp/ src/main.rs
git commit -m "feat: 创建 MCP 模块结构"
```

---

## Task 3: 定义 MCP 工具参数模型

**文件:**
- 修改: `src/mcp/models.rs`

**步骤 1: 编写参数模型测试**

创建 `tests/mcp_models_test.rs`:

```rust
use calendarsync::mcp::models::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[test]
fn test_create_event_params_schema() {
    // 验证 JSON Schema 可以生成
    let schema = schemars::schema_for!(CreateEventParams);
    assert_eq!(schema.schema_object().metadata.as_ref().unwrap().title.as_ref().unwrap(), "CreateEventParams");
}

#[test]
fn test_create_event_params_serialization() {
    let params = CreateEventParams {
        title: "测试会议".to_string(),
        description: Some("讨论 MCP 集成".to_string()),
        location: Some("会议室 A".to_string()),
        start_time: "2026-04-25T10:00:00+08:00".to_string(),
        end_time: "2026-04-25T11:00:00+08:00".to_string(),
        recurrence_rule: None,
        recurrence_until: None,
        reminder_minutes: Some(15),
        tags: Some(vec!["work".to_string(), "important".to_string()]),
    };

    let json = serde_json::to_string(&params).unwrap();
    assert!(json.contains("\"title\":\"测试会议\""));
    assert!(json.contains("\"reminder_minutes\":15"));
}
```

**步骤 2: 运行测试确认失败**

运行: `cargo test --test mcp_models_test`
预期: FAIL - "CreateEventParams not defined"

**步骤 3: 实现参数模型**

编辑 `src/mcp/models.rs`:

```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// 创建日程参数
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct CreateEventParams {
    #[schemars(description = "日程标题")]
    pub title: String,

    #[schemars(description = "日程描述")]
    pub description: Option<String>,

    #[schemars(description = "地点")]
    pub location: Option<String>,

    #[schemars(description = "开始时间 (RFC3339格式，上海时区)")]
    pub start_time: String,

    #[schemars(description = "结束时间 (RFC3339格式，上海时区)")]
    pub end_time: String,

    #[schemars(description = "重复规则 (RRULE格式)")]
    pub recurrence_rule: Option<String>,

    #[schemars(description = "重复结束时间 (RFC3339格式)")]
    pub recurrence_until: Option<String>,

    #[schemars(description = "提前提醒分钟数")]
    pub reminder_minutes: Option<i32>,

    #[schemars(description = "标签")]
    pub tags: Option<Vec<String>>,
}

/// 查询日程列表参数
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct ListEventsParams {
    #[schemars(description = "开始日期过滤 (RFC3339格式)")]
    pub from: Option<String>,

    #[schemars(description = "结束日期过滤 (RFC3339格式)")]
    pub to: Option<String>,

    #[schemars(description = "状态过滤 (active, expired, all)")]
    pub status: Option<String>,

    #[schemars(description = "关键词搜索")]
    pub keyword: Option<String>,
}

/// 获取单个日程参数
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GetEventParams {
    #[schemars(description = "日程 ID")]
    pub id: String,
}

/// 更新日程参数
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct UpdateEventParams {
    #[schemars(description = "日程 ID")]
    pub id: String,

    #[schemars(description = "日程标题")]
    pub title: Option<String>,

    #[schemars(description = "日程描述")]
    pub description: Option<String>,

    #[schemars(description = "地点")]
    pub location: Option<String>,

    #[schemars(description = "开始时间 (RFC3339格式，上海时区)")]
    pub start_time: Option<String>,

    #[schemars(description = "结束时间 (RFC3339格式，上海时区)")]
    pub end_time: Option<String>,

    #[schemars(description = "状态 (active, expired)")]
    pub status: Option<String>,

    #[schemars(description = "重复规则 (RRULE格式)")]
    pub recurrence_rule: Option<String>,

    #[schemars(description = "重复结束时间 (RFC3339格式)")]
    pub recurrence_until: Option<String>,

    #[schemars(description = "提前提醒分钟数")]
    pub reminder_minutes: Option<i32>,

    #[schemars(description = "标签")]
    pub tags: Option<Vec<String>>,
}

/// 删除日程参数
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct DeleteEventParams {
    #[schemars(description = "日程 ID")]
    pub id: String,
}
```

**步骤 4: 在 mod.rs 中导出模型**

编辑 `src/mcp/mod.rs`:

```rust
pub mod models;
pub mod transport;
pub mod server;
pub mod handlers;

pub use models::*;
pub use server::CalendarMCP;
pub use transport::create_mcp_router;
```

**步骤 5: 运行测试确认通过**

运行: `cargo test --test mcp_models_test`
预期: PASS

**步骤 6: 运行所有测试确保无破坏**

运行: `cargo test`
预期: 所有测试通过

**步骤 7: 提交**

```bash
git add src/mcp/models.rs tests/mcp_models_test.rs
git commit -m "feat: 定义 MCP 工具参数模型"
```

---

## Task 4: 实现 MCP HTTP 传输层

**文件:**
- 修改: `src/mcp/transport.rs`

**步骤 1: 编写传输层测试**

创建 `tests/mcp_transport_test.rs`:

```rust
use axum::{
    body::Body,
    http::{Method, Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn test_mcp_router_exists() {
    // 验证 MCP 路由可以创建
    use calendarsync::state::AppState;
    use calendarsync::db::repositories::{EventRepository, UserRepository, WebhookRepository};
    use calendarsync::services::WebhookService;

    // 创建测试用的 repositories (实际实现中会使用 mock)
    // 这里只验证路由可以构建
}
```

**步骤 2: 运行测试确认失败**

运行: `cargo test --test mcp_transport_test`
预期: FAIL - "create_mcp_router not defined"

**步骤 3: 实现 HTTP 传输适配器**

编辑 `src/mcp/transport.rs`:

```rust
use axum::{
    extract::{Request, State},
    response::{IntoResponse, Response},
    routing::post,
    Router,
};
use std::sync::Arc;
use crate::state::AppState;

/// 创建 MCP HTTP 路由
pub fn create_mcp_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", post(mcp_handler))
        .with_state(state)
}

/// MCP HTTP 处理器
/// 将 HTTP POST 请求转换为 MCP JSON-RPC 消息
async fn mcp_handler(
    State(state): State<Arc<AppState>>,
    req: Request,
) -> Result<Response, AppError> {
    // JSON-RPC 2.0 消息格式处理
    // 实现在 Task 5 中完成
    Ok(Response::new("MCP endpoint - coming soon".into()))
}
```

**步骤 4: 在 main.rs 中集成 MCP 路由**

编辑 `src/main.rs`:

```rust
use calendarsync::mcp;

async fn main() -> anyhow::Result<()> {
    // ... 现有代码 ...

    let mcp_routes = mcp::create_mcp_router(state.clone());

    let app = Router::new()
        .merge(public_routes)
        .merge(api_routes)
        .nest("/mcp", mcp_routes)  // 新增 MCP 路由
        .with_state(state.clone())
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    // ... 其余代码保持不变 ...
}
```

**步骤 5: 验证服务启动**

运行: `cargo run`
预期: 服务成功启动，监听端口包含 `/mcp` 路由

停止服务: Ctrl+C

**步骤 6: 测试 MCP 端点响应**

运行: `curl -X POST http://127.0.0.1:8080/mcp/ -H "Content-Type: application/json" -d '{"test": true}'`
预期: HTTP 200，响应包含 "MCP endpoint - coming soon"

**步骤 7: 提交**

```bash
git add src/mcp/transport.rs src/main.rs
git commit -m "feat: 实现 MCP HTTP 传输层框架"
```

---

## Task 5: 实现 MCP 服务器核心

**文件:**
- 修改: `src/mcp/server.rs`

**步骤 1: 编写 MCP 服务器测试**

创建 `tests/mcp_server_test.rs`:

```rust
#[test]
fn test_calendar_mcp_can_be_created() {
    use calendarsync::mcp::CalendarMCP;

    // CalendarMCP 需要可以创建
    // 实际测试会在集成测试中进行
}
```

**步骤 2: 运行测试确认失败**

运行: `cargo test --test mcp_server_test`
预期: FAIL - "CalendarMCP not defined"

**步骤 3: 实现 MCP 服务器结构**

编辑 `src/mcp/server.rs`:

```rust
use rmcp::{
    ServerHandler,
    handler::server::wrapper::Parameters,
    schemars,
    tool,
    tool_router,
    ServiceExt,
};
use std::sync::Arc;
use crate::db::repositories::{EventRepository, UserRepository};
use crate::handlers::AuthenticatedUser;
use crate::mcp::models::*;

/// Calendar MCP 服务器
#[derive(Clone)]
pub struct CalendarMCP {
    pub event_repo: Arc<EventRepository>,
    pub user_repo: Arc<UserRepository>,
    pub current_user: AuthenticatedUser,
}

impl CalendarMCP {
    pub fn new(
        event_repo: Arc<EventRepository>,
        user_repo: Arc<UserRepository>,
        current_user: AuthenticatedUser,
    ) -> Self {
        Self {
            event_repo,
            user_repo,
            current_user,
        }
    }
}

// 工具实现将在 Task 6 中完成
```

**步骤 4: 运行测试确认通过**

运行: `cargo test --test mcp_server_test`
预期: PASS

**步骤 5: 提交**

```bash
git add src/mcp/server.rs
git commit -m "feat: 实现 MCP 服务器核心结构"
```

---

## Task 6: 实现 MCP 工具处理器

**文件:**
- 修改: `src/mcp/handlers.rs`
- 修改: `src/mcp/server.rs`

**步骤 1: 编写工具处理器测试**

创建 `tests/mcp_handlers_test.rs`:

```rust
use calendarsync::mcp::{CalendarMCP, CreateEventParams, ListEventsParams};

#[tokio::test]
async fn test_list_events_tool() {
    use calendarsync::db::repositories::EventRepository;
    use calendarsync::handlers::AuthenticatedUser;
    use std::sync::Arc;

    // 使用内存数据库测试
    // 测试 list_events 工具
}
```

**步骤 2: 运行测试确认失败**

运行: `cargo test --test mcp_handlers_test`
预期: FAIL - "tool methods not defined"

**步骤 3: 实现工具处理器**

编辑 `src/mcp/handlers.rs`:

```rust
use rmcp::{
    ErrorData as McpError,
    handler::server::wrapper::Parameters,
    schemars,
    tool,
    ServerHandler,
    model::{ServerInfo, ServerCapabilities},
    service::RequestContext,
};
use crate::db::repositories::EventRepository;
use crate::handlers::AuthenticatedUser;
use crate::mcp::models::*;
use crate::models::{CreateEvent, QueryEvents, UpdateEvent};
use std::sync::Arc;

use super::server::CalendarMCP;

#[tool_router]
impl CalendarMCP {
    /// 创建日程工具
    #[tool(description = "创建新的日程")]
    fn calendar_create_event(
        &self,
        Parameters(params): Parameters<CreateEventParams>,
    ) -> Result<String, McpError> {
        // 实现将在下一步完成
        Ok("event created".to_string())
    }

    /// 查询日程列表工具
    #[tool(description = "查询日程列表")]
    fn calendar_list_events(
        &self,
        Parameters(params): Parameters<ListEventsParams>,
    ) -> Result<String, McpError> {
        Ok("events listed".to_string())
    }

    /// 获取单个日程工具
    #[tool(description = "获取单个日程详情")]
    fn calendar_get_event(
        &self,
        Parameters(params): Parameters<GetEventParams>,
    ) -> Result<String, McpError> {
        Ok("event retrieved".to_string())
    }

    /// 更新日程工具
    #[tool(description = "更新已有日程")]
    fn calendar_update_event(
        &self,
        Parameters(params): Parameters<UpdateEventParams>,
    ) -> Result<String, McpError> {
        Ok("event updated".to_string())
    }

    /// 删除日程工具
    #[tool(description = "删除日程")]
    fn calendar_delete_event(
        &self,
        Parameters(params): Parameters<DeleteEventParams>,
    ) -> Result<String, McpError> {
        Ok("event deleted".to_string())
    }
}
```

**步骤 4: 实现 ServerHandler trait**

编辑 `src/mcp/server.rs`:

```rust
use rmcp::{
    ServerHandler,
    model::{ServerInfo, ServerCapabilities},
};

impl ServerHandler for CalendarMCP {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}
```

**步骤 5: 在 mod.rs 中导出 handlers**

编辑 `src/mcp/mod.rs`:

```rust
pub mod models;
pub mod transport;
pub mod server;
pub mod handlers;

pub use models::*;
pub use server::CalendarMCP;
pub use transport::create_mcp_router;
```

**步骤 6: 运行测试确认编译通过**

运行: `cargo check`
预期: 编译通过

**步骤 7: 提交**

```bash
git add src/mcp/handlers.rs src/mcp/server.rs src/mcp/mod.rs
git commit -m "feat: 实现 MCP 工具处理器框架"
```

---

## Task 7: 实现创建日程工具

**文件:**
- 修改: `src/mcp/handlers.rs`

**步骤 1: 编写创建日程集成测试**

创建 `tests/integration/mcp_create_event_test.rs`:

```rust
use calendarsync::mcp::{CalendarMCP, CreateEventParams};
use calendarsync::db::{create_pool, run_migrations};
use calendarsync::db::repositories::{EventRepository, UserRepository};
use calendarsync::handlers::{hash_password, AuthenticatedUser};
use rmcp::handler::server::wrapper::Parameters;

#[tokio::test]
async fn test_mcp_create_event() {
    // 使用内存数据库
    let db_path = format!("sqlite::file:///tmp/mcp-test-{}.db", uuid::Uuid::new_v4());
    let pool = create_pool(&db_path).await.unwrap();
    run_migrations(&pool).await.unwrap();

    let user_repo = Arc::new(UserRepository::new(pool.clone()));
    let event_repo = Arc::new(EventRepository::new(pool));

    // 创建测试用户
    let password_hash = hash_password("test_password").unwrap();
    let user = user_repo.create(
        calendarsync::models::CreateUser {
            username: "test_user".to_string(),
            password: None,
            is_admin: Some(false),
        },
        Some(password_hash),
    ).await.unwrap();

    let auth_user = AuthenticatedUser {
        user_id: user.id.clone(),
        username: user.username,
        is_admin: user.is_admin,
    };

    let mcp = CalendarMCP::new(event_repo, user_repo, auth_user);

    // 测试创建日程
    let params = CreateEventParams {
        title: "测试会议".to_string(),
        description: Some("MCP 测试".to_string()),
        location: Some("线上".to_string()),
        start_time: "2026-04-25T10:00:00+08:00".to_string(),
        end_time: "2026-04-25T11:00:00+08:00".to_string(),
        recurrence_rule: None,
        recurrence_until: None,
        reminder_minutes: Some(15),
        tags: Some(vec!["test".to_string()]),
    };

    let result = mcp.calendar_create_event(Parameters(params));
    assert!(result.is_ok());

    let event_id = result.unwrap();
    assert!(!event_id.is_empty());
}
```

**步骤 2: 运行测试确认失败**

运行: `cargo test --test mcp_create_event_test`
预期: FAIL - "返回的是占位字符串，不是真实 event_id"

**步骤 3: 实现创建日程逻辑**

编辑 `src/mcp/handlers.rs`:

```rust
#[tool(description = "创建新的日程")]
fn calendar_create_event(
    &self,
    Parameters(params): Parameters<CreateEventParams>,
) -> Result<String, McpError> {
    // 验证时间格式
    if params.start_time.is_empty() {
        return Err(McpError::invalid_params(
            "start_time is required",
            None,
        ));
    }

    if params.end_time.is_empty() {
        return Err(McpError::invalid_params(
            "end_time is required",
            None,
        ));
    }

    // 转换为内部 CreateEvent
    let create_event = CreateEvent {
        title: params.title,
        description: params.description,
        location: params.location,
        start_time: params.start_time,
        end_time: params.end_time,
        recurrence_rule: params.recurrence_rule,
        recurrence_until: params.recurrence_until,
        reminder_minutes: params.reminder_minutes,
        tags: params.tags.map(|t| serde_json::to_string(&t).unwrap()),
    };

    // 使用 tokio runtime 在同步上下文中运行异步代码
    let event_repo = self.event_repo.clone();
    let user_id = self.current_user.user_id.clone();

    let event = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async move {
            event_repo
                .create(user_id, create_event)
                .await
                .map_err(|e| McpError::internal_error(format!("Failed to create event: {}", e), None))
        })
    })?;

    Ok(event.id)
}
```

**步骤 4: 运行测试确认通过**

运行: `cargo test --test mcp_create_event_test`
预期: PASS

**步骤 5: 提交**

```bash
git add src/mcp/handlers.rs tests/integration/mcp_create_event_test.rs
git commit -m "feat: 实现 MCP 创建日程工具"
```

---

## Task 8: 实现查询日程列表工具

**文件:**
- 修改: `src/mcp/handlers.rs`

**步骤 1: 编写查询列表集成测试**

创建 `tests/integration/mcp_list_events_test.rs`:

```rust
#[tokio::test]
async fn test_mcp_list_events() {
    // 类似 Task 7，测试 list_events 工具
    // 1. 创建测试用户和多个日程
    // 2. 调用 calendar_list_events
    // 3. 验证返回的日程列表
}
```

**步骤 2: 运行测试确认失败**

运行: `cargo test --test mcp_list_events_test`
预期: FAIL

**步骤 3: 实现查询列表逻辑**

编辑 `src/mcp/handlers.rs`:

```rust
#[tool(description = "查询日程列表")]
fn calendar_list_events(
    &self,
    Parameters(params): Parameters<ListEventsParams>,
) -> Result<String, McpError> {
    let event_repo = self.event_repo.clone();
    let user_id = self.current_user.user_id.clone();

    let query = QueryEvents {
        user_id: Some(user_id),
        status: params.status,
        from: params.from,
        to: params.to,
        keyword: params.keyword,
    };

    let events = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async move {
            event_repo
                .find_by_user(&user_id, query)
                .await
                .map_err(|e| McpError::internal_error(format!("Failed to list events: {}", e), None))
        })
    })?;

    // 转换为 JSON 字符串返回
    serde_json::to_string(&events)
        .map_err(|e| McpError::internal_error(format!("Failed to serialize events: {}", e), None))
}
```

**步骤 4: 运行测试确认通过**

运行: `cargo test --test mcp_list_events_test`
预期: PASS

**步骤 5: 提交**

```bash
git add src/mcp/handlers.rs tests/integration/mcp_list_events_test.rs
git commit -m "feat: 实现 MCP 查询日程列表工具"
```

---

## Task 9: 实现获取单个日程工具

**文件:**
- 修改: `src/mcp/handlers.rs`

**步骤 1-5:** 类似 Task 7 和 8，实现 `calendar_get_event` 工具

**提交:**

```bash
git commit -m "feat: 实现 MCP 获取单个日程工具"
```

---

## Task 10: 实现更新日程工具

**文件:**
- 修改: `src/mcp/handlers.rs`

**步骤 1-5:** 实现更新日程工具，支持部分字段更新

**提交:**

```bash
git commit -m "feat: 实现 MCP 更新日程工具"
```

---

## Task 11: 实现删除日程工具

**文件:**
- 修改: `src/mcp/handlers.rs`

**步骤 1-5:** 实现删除日程工具

**提交:**

```bash
git commit -m "feat: 实现 MCP 删除日程工具"
```

---

## Task 12: 实现 API Key 认证集成

**文件:**
- 修改: `src/mcp/transport.rs`

**步骤 1: 编写认证测试**

创建 `tests/mcp_auth_test.rs`:

```rust
#[tokio::test]
async fn test_mcp_requires_api_key() {
    // 测试没有 API Key 的请求被拒绝
    // 测试有效 API Key 的请求被接受
}
```

**步骤 2-3:** 实现认证中间件并测试

**提交:**

```bash
git commit -m "feat: 实现 MCP API Key 认证"
```

---

## Task 13: 实现错误处理与映射

**文件:**
- 修改: `src/mcp/handlers.rs`
- 创建: `src/mcp/error.rs`

**步骤 1-5:** 实现 AppError 到 McpError 的转换

**提交:**

```bash
git commit -m "feat: 实现 MCP 错误处理与映射"
```

---

## Task 14: 实现完整 JSON-RPC 消息处理

**文件:**
- 修改: `src/mcp/transport.rs`

**步骤 1-5:** 完成完整的 JSON-RPC 2.0 消息解析和路由

**提交:**

```bash
git commit -m "feat: 实现完整 JSON-RPC 消息处理"
```

---

## Task 15: 端到端集成测试

**文件:**
- 创建: `tests/e2e/mcp_e2e_test.rs`

**步骤 1: 编写完整流程测试**

```rust
#[tokio::test]
async fn test_mcp_full_workflow() {
    // 1. 连接到 MCP 服务器
    // 2. 列出可用工具
    // 3. 创建日程
    // 4. 查询日程
    // 5. 更新日程
    // 6. 删除日程
}
```

**步骤 2-5:** 实现并验证

**提交:**

```bash
git commit -m "test: 添加 MCP 端到端集成测试"
```

---

## Task 16: 更新文档

**文件:**
- 修改: `CLAUDE.md`
- 修改: `README.md`

**步骤 1: 更新 CLAUDE.md 添加 MCP 接口说明**

**步骤 2: 更新 README.md 添加使用示例**

**步骤 3: 验证文档格式**

运行: `cargo doc --no-deps --open`
预期: 文档成功生成

**步骤 4: 提交**

```bash
git add CLAUDE.md README.md
git commit -m "docs: 添加 MCP 接口文档和使用示例"
```

---

## Task 17: 最终验证与发布准备

**步骤 1: 运行完整测试套件**

运行: `cargo test --all`
预期: 所有测试通过

**步骤 2: 运行 Clippy 检查**

运行: `cargo clippy -- -D warnings`
预期: 无警告

**步骤 3: 检查代码格式**

运行: `cargo fmt --check`
预期: 无格式问题

**步骤 4: 构建发布版本**

运行: `cargo build --release`
预期: 成功构建

**步骤 5: 启动服务并手动测试**

运行: `cargo run`
预期: 服务正常运行，可以通过 MCP 客户端调用工具

**步骤 6: 创建发布标签**

```bash
git tag -a v0.2.0 -m "添加 MCP HTTP 接口支持"
git push origin v0.2.0
```

---

## 实现注意事项

### TDD 原则
- 每个功能先写测试
- 运行测试确认失败
- 实现最小代码使测试通过
- 重构优化
- 频繁提交

### 关键依赖
- rmcp SDK 文档: https://github.com/modelcontextprotocol/rust-sdk
- MCP 规范: https://modelcontextprotocol.io
- 现有 API: 参考 `src/handlers/events.rs`

### 潜在问题与解决方案
1. **同步/异步混合**: MCP 工具是同步的，Repository 是异步的
   - 解决: 使用 `tokio::task::block_in_place`

2. **JSON Schema 生成**: MCP 需要参数的 JSON Schema
   - 解决: 使用 `schemars` derive macro

3. **认证传递**: 需要将 API Key 认证结果传递给 MCP 工具
   - 解决: 在 `CalendarMCP` 结构中存储 `AuthenticatedUser`

### 测试策略
- 单元测试: 每个工具的逻辑
- 集成测试: 工具与数据库交互
- 端到端测试: 完整的 MCP JSON-RPC 流程
