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

### 认证

所有 API 请求需在 Header 中携带 `X-API-Key`。

### 用户管理

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | /api/users | 创建用户（需管理员） |
| GET | /api/users | 用户列表（需管理员） |
| GET | /api/users/:id | 获取用户详情（需管理员） |
| PUT | /api/users/:id | 更新用户（需管理员） |
| DELETE | /api/users/:id | 删除用户（需管理员） |

### 日程管理

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | /api/events | 创建日程 |
| GET | /api/events | 日程列表 |
| GET | /api/events/search | 搜索日程 |
| GET | /api/events/:id | 获取日程详情 |
| PUT | /api/events/:id | 更新日程 |
| DELETE | /api/events/:id | 删除日程 |

### Webhook 管理

| 方法 | 路径 | 说明 |
|------|------|------|
| POST | /api/webhooks | 创建 Webhook |
| GET | /api/webhooks | Webhook 列表 |
| GET | /api/webhooks/:id | 获取 Webhook 详情 |
| PUT | /api/webhooks/:id | 更新 Webhook |
| DELETE | /api/webhooks/:id | 删除 Webhook |

### 日历订阅

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | /calendar/:user_id/subscribe.ics | 订阅用户日历 |

## 配置

编辑 `config.toml` 文件：

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

## License

MIT
