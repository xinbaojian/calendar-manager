# CalendarSync - 项目上下文

## 项目概述

CalendarSync 是一个用 Rust 编写的轻量级日程管理服务，支持多用户、RESTful API、iPhone 日历订阅（.ics 格式）、重复日程、Webhook 通知以及 Web 管理界面。

### 技术栈

- **语言**: Rust (Edition 2021)
- **Web 框架**: Axum 0.7 + Tokio 异步运行时
- **数据库**: SQLite + sqlx（异步）
- **模板引擎**: Askama（用于 Web 管理界面）
- **认证**: API Key + JWT（JSON Web Token）
- **密码哈希**: Argon2
- **日历格式**: iCalendar (.ics) RFC 5545 兼容

### 架构概览

项目采用清晰的分层架构：

- **`src/handlers/`** — HTTP 请求处理器（Axum），包含认证中间件
- **`src/db/repositories/`** — 数据库访问层，使用 Repository 模式
- **`src/models/`** — 领域模型（User, Event, Webhook）
- **`src/services/`** — 业务逻辑（Webhook 投递服务）
- **`src/ical/`** — iCalendar (.ics) 生成器，用于日历订阅
- **`src/config.rs`** — 配置加载
- **`src/error.rs`** — 集中式错误处理（AppError）
- **`src/state.rs`** — 应用状态（Arc-wrapped repositories）
- **`src/templates.rs`** — Askama 模板处理
- **`templates/`** — HTML 模板（Web 管理界面）
- **`migrations/`** — 数据库迁移脚本

## 数据库

- 使用 **SQLite** + sqlx 进行异步数据库操作
- 启动时自动运行迁移
- 所有时间戳使用 **Asia/Shanghai (UTC+8)** 时区存储
- 外键级联删除（users → events/webhooks）

### 主要表结构

- `users` — 用户表（含 API Key、密码哈希、管理员标志）
- `events` — 日程表（支持重复规则 RRULE、标签、状态）
- `webhooks` — Webhook 配置表
- `webhook_logs` — Webhook 投递日志表

## 认证机制

两种认证方式：

1. **API Key** — 请求头 `X-API-Key`，用于程序化访问
2. **JWT Token** — 基于密码的登录，用于 Web UI

管理员用户在首次启动时自动创建，凭据来自 `config.toml`。

## 路由结构

| 前缀 | 说明 | 认证 |
|------|------|------|
| `/` | 首页 | 公开 |
| `/events`, `/settings`, `/webhooks`, `/users` | SPA 回退路由 → index.html | 公开（前端路由） |
| `/api/auth/login` | 登录 | 公开 |
| `/api/auth/change-password` | 修改密码 | 已认证 |
| `/api/auth/api-key` | 获取/重置 API Key | 已认证 |
| `/api/users/*` | 用户管理（仅管理员） | 已认证 + 管理员 |
| `/api/events/*` | 日程 CRUD | 已认证 |
| `/api/webhooks/*` | Webhook CRUD | 已认证 |
| `/calendar/:user_id/subscribe.ics` | 日历订阅 | 公开 |

## Webhook 系统

- 异步投递，支持指数退避重试
- 支持 HMAC-SHA256 签名验证（`X-Webhook-Signature` 头）
- 可配置超时和最大重试次数

## 时区处理

**关键**: 所有时间戳统一使用 **Asia/Shanghai (UTC+8)** 时区：
- 数据库存储：RFC3339 字符串，带上海时区
- iCal 生成：显式 TZID=Asia/Shanghai
- 定时清理：使用上海时区进行过期检查

## 配置

配置文件路径可通过 `CONFIG_PATH` 环境变量指定，默认为 `config.toml`。

```toml
[server]
host = "127.0.0.1"
port = 8080

[database]
path = "./data/calendar.db"

[auth]
admin_username = "admin"
admin_api_key = "MUST_CHANGE_THIS"      # 必须修改
admin_password = "changeme"              # 必须修改
jwt_secret = "MUST_CHANGE_THIS"          # 必须修改
jwt_exp_hours = 24

[cleanup]
check_interval_hours = 1
auto_delete_expired_days = 210

[webhook]
timeout_seconds = 10
max_retries = 3
```

> ⚠️ **切勿提交 `config.toml`** — 包含敏感密钥，使用 `config.example.toml` 代替。

## 开发与构建命令

### 运行应用

```bash
# 默认配置运行
cargo run

# 自定义配置路径
CONFIG_PATH=/path/to/config.toml cargo run
```

### 测试

```bash
# 运行所有测试
cargo test

# 运行特定测试模块
cargo test api_tests

# 显示测试输出
cargo test -- --nocapture
```

测试使用内存 SQLite 数据库（`/tmp/calendarsync-test-{uuid}.db`），每个测试隔离运行。

### Docker 构建

```bash
# 本地构建
make build

# 跨平台构建 (linux/amd64)
make buildx

# 构建并推送
make push

# 发布版本 (make release VERSION=1.0.0)
make release VERSION=1.0.0
```

### 部署

```bash
# docker-compose 一键部署
docker-compose up -d

# 远程部署（需配置 SSH_HOST 和 DEPLOY_DIR）
make deploy SSH_HOST=root@your-server DEPLOY_DIR=/opt/calendar-sync
```

### 定时清理任务

后台任务每隔 `check_interval_hours` 小时执行：
1. 将过期事件的 `status` 标记为 `expired`
2. 删除超过 `auto_delete_expired_days` 天的过期事件

### 重复规则 (RRULE)

支持 iCal 兼容的重复规则：

| 规则 | 说明 |
|------|------|
| `FREQ=DAILY` | 每天 |
| `FREQ=WEEKLY` | 每周 |
| `FREQ=MONTHLY` | 每月 |
| `FREQ=YEARLY` | 每年 |
| `FREQ=WEEKLY;BYDAY=MO,WE,FR` | 周一、三、五 |
| `FREQ=DAILY;INTERVAL=2` | 每 2 天 |
| `FREQ=DAILY;COUNT=10` | 10 次后结束 |
| `FREQ=DAILY;UNTIL=20251231T235959Z` | 直到指定日期 |

## 错误处理

所有错误实现 `IntoResponse` trait，返回统一 JSON 格式：

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "人类可读的错误信息",
    "details": {}  // 可选
  }
}
```

## 重要注意事项

1. **切勿提交 config.toml** — 包含密钥，使用 config.example.toml
2. **始终使用上海时区** 处理时间戳
3. **API Key 验证在中间件中进行** — handler 接收 AuthenticatedUser
4. **Repository 方法返回 AppResult<T>** — 集中式错误处理
5. **iCal 行折叠** — 超过 75 字节的行按 RFC 5545 折叠

## 交流偏好

- 所有输出使用**简体中文**
