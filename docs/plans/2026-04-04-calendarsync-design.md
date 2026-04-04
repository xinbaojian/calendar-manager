# CalendarSync 系统设计文档

**日期**: 2026-04-04
**版本**: v1.0
**架构**: 单体模块化架构

---

## 一、项目概述

### 1.1 目标
为家庭小团队（2-5人）提供轻量级日程管理服务，支持：
- 多用户独立管理
- iPhone 日历订阅（.ics 格式）
- 重复日程支持
- Webhook 通知提醒
- Web 可视化管理界面

### 1.2 技术栈
- 语言: Rust
- Web框架: Axum 0.7
- 数据库: SQLite + SQLx
- 模板引擎: Askama (类型安全)
- 异步运行时: Tokio
- HTTP 客户端: Reqwest
- 部署: Docker + 云服务器

---

## 二、整体架构

```
┌─────────────────────────────────────────────────────────┐
│                    CalendarSync                        │
├─────────────────────────────────────────────────────────┤
│  入口层: Axum HTTP Server (:8080)                       │
│  接口层: REST API + Web UI + iCalendar + Webhook        │
│  业务层: UserService + EventService + CalendarService   │
│  数据层: SQLite + 清理调度器                             │
└─────────────────────────────────────────────────────────┘
```

**部署架构**: Nginx(HTTPS:443) → CalendarSync:8080(HTTP)

---

## 三、数据模型

### 3.1 表结构

#### users 表
```sql
CREATE TABLE users (
    id TEXT PRIMARY KEY,              -- usr_xxx
    username TEXT NOT NULL UNIQUE,    -- 用户名
    api_key TEXT NOT NULL UNIQUE,     -- 用户专属 API Key
    is_admin BOOLEAN DEFAULT 0,       -- 是否管理员
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
```

#### events 表
```sql
CREATE TABLE events (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    location TEXT,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    recurrence_rule TEXT,             -- RRULE
    recurrence_until TEXT,            -- 重复结束时间
    reminder_minutes INTEGER,
    tags TEXT,                        -- JSON 数组
    status TEXT DEFAULT 'active',     -- active/expired/deleted
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
CREATE INDEX idx_events_user_time ON events(user_id, start_time);
```

#### webhooks 表
```sql
CREATE TABLE webhooks (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    url TEXT NOT NULL,
    events TEXT NOT NULL,             -- JSON: ["reminder", "created"]
    secret TEXT,                      -- HMAC 签名密钥
    is_active BOOLEAN DEFAULT 1,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id)
);
```

#### webhook_logs 表
```sql
CREATE TABLE webhook_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    webhook_id TEXT NOT NULL,
    event_type TEXT NOT NULL,
    payload TEXT,
    status_code INTEGER,
    response_body TEXT,
    sent_at TEXT NOT NULL
);
```

---

## 四、API 接口

### 4.1 认证机制
```
X-API-Key: <secret>
X-User-ID: <user_id>  // admin 可操作其他用户
```

### 4.2 接口列表

#### 用户管理
```
POST   /api/users                    # 创建用户
GET    /api/users                    # 列出用户 (需 admin)
GET    /api/users/:id                # 获取用户详情
PUT    /api/users/:id                # 更新用户
DELETE /api/users/:id                # 删除用户
```

#### 日程管理
```
POST   /api/events                   # 创建日程
GET    /api/events                   # 查询日程
GET    /api/events/:id               # 获取单个日程
PUT    /api/events/:id               # 更新日程
DELETE /api/events/:id               # 删除日程
GET    /api/events/search            # 搜索历史日程
```

#### Webhook 管理
```
POST   /api/webhooks                 # 创建 Webhook
GET    /api/webhooks                 # 列出 Webhooks
PUT    /api/webhooks/:id             # 更新 Webhook
DELETE /api/webhooks/:id             # 删除 Webhook
```

#### 日历订阅
```
GET    /calendar/:user_id/subscribe.ics   # 用户专属订阅
```

#### Web UI
```
GET    /                             # 首页
GET    /events                       # 日程管理页
GET    /settings                     # 设置页
```

### 4.3 重复日程 RRULE 格式
```json
{
  "recurrence_rule": "FREQ=DAILY;INTERVAL=1",
  "recurrence_until": "2026-12-31T23:59:59+08:00"
}
```
支持频率: `DAILY`、`WEEKLY`、`MONTHLY`

---

## 五、核心业务流程

### 5.1 创建重复日程
1. 接收请求 → 验证 RRULE 格式
2. 计算重复实例 (直到 recurrence_until)
3. 批量插入 events 表
4. 返回创建的实例列表

### 5.2 订阅链接生成
1. 查询用户所有 active 日程
2. 展开重复日程的未过期实例
3. 生成 iCalendar 格式 (RFC 5545)
4. 返回 text/calendar 响应

### 5.3 定时清理
- 每小时执行一次
- 将 `end_time < now` 的日程标记为 `expired`
- 触发 Webhook 通知
- 30 天后物理删除

### 5.4 Webhook 通知
1. 查询用户配置的 Webhooks
2. 筛选匹配事件类型
3. 异步发送 POST 请求 (带 HMAC 签名)
4. 记录发送结果
5. 失败重试 (最多 3 次，指数退避)

---

## 六、错误处理

### 6.1 错误响应格式
```json
{
  "error": {
    "code": "INVALID_API_KEY",
    "message": "提供的 API Key 无效",
    "details": {}
  }
}
```

### 6.2 错误码

| 错误码 | HTTP状态 | 说明 |
|--------|----------|------|
| INVALID_API_KEY | 401 | API Key 无效 |
| INSUFFICIENT_PERMISSION | 403 | 权限不足 |
| USER_NOT_FOUND | 404 | 用户不存在 |
| EVENT_NOT_FOUND | 404 | 日程不存在 |
| INVALID_RECURRENCE_RULE | 400 | 重复规则格式错误 |
| DUPLICATE_USERNAME | 409 | 用户名已存在 |
| WEBHOOK_DELIVERY_FAILED | 500 | Webhook 发送失败 |

---

## 七、安全措施

1. **API Key**: UUID v4 自动生成
2. **Webhook 签名**: HMAC SHA256
3. **SQL 注入防护**: 参数化查询 (SQLx)
4. **输入验证**: 严格格式验证
5. **HTTPS**: Nginx SSL 终结
6. **限流**: 每 IP 每分钟 100 次请求

---

## 八、项目结构

```
calendarsync/
├── Cargo.toml
├── config.toml
├── Dockerfile
├── docker-compose.yml
├── migrations/
│   └── schema.sql
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── error.rs
│   ├── db/
│   │   ├── mod.rs
│   │   ├── pool.rs
│   │   └── repositories/
│   │       ├── user.rs
│   │       ├── event.rs
│   │       └── webhook.rs
│   ├── models/
│   │   ├── user.rs
│   │   ├── event.rs
│   │   └── webhook.rs
│   ├── services/
│   │   ├── user.rs
│   │   ├── event.rs
│   │   ├── calendar.rs
│   │   └── webhook.rs
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── auth.rs
│   │   ├── users.rs
│   │   ├── events.rs
│   │   ├── webhooks.rs
│   │   └── web.rs
│   ├── templates/
│   │   └── *.askama
│   └── ical/
│       └── generator.rs
└── tests/
    └── integration/
```

---

## 九、资源预估

| 项目 | 占用 |
|------|------|
| Rust 程序 | ~15MB |
| Alpine 基础镜像 | ~5MB |
| 容器运行时 | ~5MB |
| **合计** | ~25-30MB |
| 内存运行时 | < 50MB |
