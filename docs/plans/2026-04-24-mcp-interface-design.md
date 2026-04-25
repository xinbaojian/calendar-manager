# MCP HTTP 接口设计文档

**日期**: 2026-04-24
**方案**: 使用官方 rmcp SDK
**状态**: 设计阶段

## 1. 概述

为 CalendarSync 项目添加 Model Context Protocol (MCP) HTTP 接口，使其他 AI 工具能够通过标准协议调用日程管理功能。

### 范围

- **功能**: 日程管理 CRUD (创建、查询、更新、删除)
- **认证**: 复用现有 API Key 机制
- **部署**: 集成到现有 Axum 服务中

### 不包含

- 用户管理 MCP 接口
- Webhook 管理 MCP 接口
- iCal 订阅 MCP 接口
- SSE/实时通知

## 2. 架构设计

### 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│                        Axum Server                           │
├─────────────────────────────────────────────────────────────┤
│  ┌───────────────┐  ┌───────────────┐  ┌────────────────┐   │
│  │   REST API    │  │  MCP Server   │  │  iCal Subscribe │   │
│  │   /api/*      │  │   /mcp/*      │  │   /calendar/*   │   │
│  └───────────────┘  └───────────────┘  └────────────────┘   │
│         │                   │                  │             │
│         ▼                   ▼                  ▼             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                    Auth Middleware                     │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  Repositories & Services                     │
│  EventRepository │ UserRepository │ WebhookService          │
└─────────────────────────────────────────────────────────────┘
```

### 新增模块

| 模块 | 文件 | 职责 |
|------|------|------|
| MCP 传输层 | `src/mcp/transport.rs` | Axum HTTP MCP 传输适配器 |
| MCP 服务器 | `src/mcp/server.rs` | CalendarMCP 服务器主结构 |
| 工具处理器 | `src/mcp/handlers.rs` | MCP 工具实现 |
| 类型定义 | `src/mcp/models.rs` | MCP 请求/响应模型 |

## 3. MCP 工具定义

### 工具列表

| 工具名 | 方法 | 描述 |
|--------|------|------|
| `calendar_create_event` | create_event | 创建新日程 |
| `calendar_list_events` | list_events | 查询日程列表 |
| `calendar_get_event` | get_event | 获取单个日程详情 |
| `calendar_update_event` | update_event | 更新日程 |
| `calendar_delete_event` | delete_event | 删除日程 |

### 工具参数

```rust
// 创建日程
struct CreateEventParams {
    title: String,
    description: Option<String>,
    location: Option<String>,
    start_time: String,  // RFC3339
    end_time: String,    // RFC3339
    recurrence_rule: Option<String>,
    recurrence_until: Option<String>,
    reminder_minutes: Option<i32>,
    tags: Option<Vec<String>>,
}

// 查询日程
struct ListEventsParams {
    from: Option<String>,
    to: Option<String>,
    status: Option<String>,
    keyword: Option<String>,
}
```

## 4. 认证与授权

### 认证机制

MCP 接口使用与 REST API 相同的 API Key 认证：

```
请求头:
X-API-Key: <user_api_key>
```

### 认证流程

1. MCP 请求携带 `X-API-Key` header
2. 中间件验证 API Key
3. 从数据库获取对应用户
4. 工具处理器使用 `AuthenticatedUser` 执行操作
5. 所有操作限定在用户自己的数据范围内

## 5. 数据流

### 工具调用流程

```
Client                 MCP Server            Repository
  │                        │                      │
  │ call_tool()            │                      │
  │───────────────────────>│                      │
  │                        │ find_by_id()         │
  │                        │─────────────────────>│
  │                        │ Event                │
  │                        │<─────────────────────│
  │                        │ [async] webhook      │
  │ ToolResult             │                      │
  │<───────────────────────│                      │
```

### Webhook 触发

工具执行成功后，异步触发 webhook 通知（与 REST API 一致）。

## 6. 错误处理

### 错误映射

| AppError | MCP Error | Code |
|----------|-----------|------|
| NotFound | resource_not_found | -32700 |
| Unauthorized | unauthorized | -32701 |
| ValidationError | invalid_params | -32602 |
| Internal | internal_error | -32603 |

### 错误响应格式

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid params: start_time is required",
    "data": {
      "field": "start_time",
      "reason": "missing"
    }
  }
}
```

## 7. 依赖更新

```toml
[dependencies]
rmcp = { version = "0.16", features = ["server"] }
schemars = "0.8"
```

## 8. 测试策略

### 测试层级

1. **单元测试**: 工具处理器的输入验证和逻辑
2. **集成测试**: 端到端 MCP 调用流程
3. **协议测试**: MCP conformance suite

### 测试配置

```toml
[dev-dependencies]
rmcp = { version = "0.16", features = ["client"] }
```

## 9. 配置与部署

### 配置文件

在现有 `config.toml` 中添加：

```toml
[mcp]
enabled = true
```

### 路由集成

在 `src/main.rs` 中：

```rust
mod mcp;

let mcp_routes = mcp::create_router(state.clone());

let app = Router::new()
    .merge(public_routes)
    .merge(api_routes)
    .nest("/mcp", mcp_routes)  // MCP 端点
    .with_state(state);
```

## 10. 实现清单

- [ ] 添加依赖 (rmcp, schemars)
- [ ] 创建 MCP 模块结构
- [ ] 实现 HTTP 传输层适配器
- [ ] 实现 CalendarMCP 服务器
- [ ] 实现工具处理器 (create, list, get, update, delete)
- [ ] 实现 API Key 认证集成
- [ ] 错误处理与映射
- [ ] 单元测试
- [ ] 集成测试
- [ ] 文档更新

## 11. 参考资源

- [MCP 规范](https://modelcontextprotocol.io)
- [rmcp SDK 文档](https://github.com/modelcontextprotocol/rust-sdk)
- [现有 API 文档](CLAUDE.md)
