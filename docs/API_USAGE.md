# CalendarSync API 使用手册

CalendarSync 是一个日程管理服务，提供 RESTful API 用于程序的日程增删改查、日历订阅和 Webhook 通知。
## API key

eHIuzKcqbWYL4DfPDkBMhqhtgBtuekE3wTLkCOhr5FgxAISlGRdSseBaAR4W6Qu8

## 基础信息

| 项目 | 值 |
|------|---|
| Base URL | `https://ics.xiuyuan.xin` |
| 协议 | HTTPS |
| 数据格式 | JSON |
| 时间格式 | ISO 8601 / RFC 3339（如 `2026-04-04T15:00:00+08:00`） |
| 字符编码 | UTF-8 |

---

## 认证方式

所有 `/api/` 接口（登录接口除外）均需要认证。支持两种方式：

### 方式一：API Key（推荐用于程序调用）

在请求头中携带 API Key：

```
X-API-Key: your-api-key-here
```

### 方式二：JWT Token

先通过登录接口获取 Token，然后在请求头中携带：

```
Authorization: Bearer eyJhbGciOiJIUzI1NiJ9...
```

Token 有效期默认 24 小时，过期后需重新登录获取。

---

## 错误响应格式

所有错误均返回统一格式：

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "错误描述信息"
  }
}
```

常见 HTTP 状态码：

| 状态码 | 含义 |
|--------|------|
| 200 | 成功 |
| 201 | 创建成功 |
| 204 | 删除成功（无返回体） |
| 400 | 请求参数错误 |
| 401 | 未认证 / 认证失败 |
| 403 | 权限不足 |
| 404 | 资源不存在 |
| 409 | 资源冲突（如用户名重复） |

---

## 接口列表

### 1. 用户认证

#### 1.1 登录

```
POST /api/auth/login
```

无需认证。

**请求体：**
```json
{
  "username": "admin",
  "password": "your-password"
}
```

**响应：**
```json
{
  "token": "eyJhbGciOiJIUzI1NiJ9...",
  "user": {
    "id": "usr_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
    "username": "admin",
    "is_admin": true
  }
}
```

#### 1.2 修改密码

```
POST /api/auth/change-password
```

**请求体：**
```json
{
  "current_password": "old-password",
  "new_password": "new-password"
}
```

**响应：**
```json
{
  "message": "密码修改成功"
}
```

#### 1.3 获取 API Key

```
GET /api/auth/api-key
```

返回当前用户的 API Key。

**响应：**
```json
{
  "api_key": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
}
```

#### 1.4 重新生成 API Key

```
POST /api/auth/api-key
```

旧 Key 立即失效，返回新 Key。

**响应：**
```json
{
  "api_key": "yyyyyyyy-yyyy-yyyy-yyyy-yyyyyyyyyyyy"
}
```

---

### 2. 日程管理

#### 2.1 创建日程

```
POST /api/events
```

**请求体：**
```json
{
  "user_id": "usr_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "event": {
    "title": "项目周会",
    "description": "讨论本周进度",
    "location": "会议室A",
    "start_time": "2026-04-07T09:00:00+08:00",
    "end_time": "2026-04-07T10:00:00+08:00",
    "reminder_minutes": 15,
    "tags": ["工作", "周会"]
  }
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `user_id` | string | 是 | 日程所属用户 ID |
| `event.title` | string | 是 | 日程标题，不能为空 |
| `event.description` | string | 否 | 描述 |
| `event.location` | string | 否 | 地点 |
| `event.start_time` | string | 是 | 开始时间（RFC 3339） |
| `event.end_time` | string | 是 | 结束时间，必须晚于开始时间 |
| `event.recurrence_rule` | string | 否 | 重复规则（RRULE 格式） |
| `event.recurrence_until` | string | 否 | 重复结束时间 |
| `event.reminder_minutes` | integer | 否 | 提前提醒分钟数 |
| `event.tags` | string[] | 否 | 标签列表 |

**响应（201）：**
```json
{
  "id": "evt_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "user_id": "usr_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "title": "项目周会",
  "description": "讨论本周进度",
  "location": "会议室A",
  "start_time": "2026-04-07T09:00:00+08:00",
  "end_time": "2026-04-07T10:00:00+08:00",
  "recurrence_rule": null,
  "recurrence_until": null,
  "reminder_minutes": 15,
  "tags": ["工作", "周会"],
  "status": "active",
  "created_at": "2026-04-04T15:00:00+00:00"
}
```

#### 2.2 查询日程列表

```
GET /api/events?user_id=xxx&status=active&from=2026-04-01T00:00:00+08:00&to=2026-04-30T23:59:59+08:00&keyword=周会
```

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `user_id` | string | 否 | 用户 ID（默认为当前用户） |
| `status` | string | 否 | 筛选状态：`active` / `cancelled` / `expired` |
| `from` | string | 否 | 起始时间筛选 |
| `to` | string | 否 | 截止时间筛选 |
| `keyword` | string | 否 | 关键词搜索（匹配标题和描述） |

**响应：**
```json
[
  {
    "id": "evt_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
    "title": "项目周会",
    "status": "active",
    "start_time": "2026-04-07T09:00:00+08:00",
    "end_time": "2026-04-07T10:00:00+08:00",
    "...": "..."
  }
]
```

#### 2.3 获取单个日程

```
GET /api/events/:id
```

**响应：** 同上单个日程对象。

#### 2.4 更新日程

```
PUT /api/events/:id
```

仅传入需要更新的字段，未传入的字段保持不变。

**请求体：**
```json
{
  "title": "项目周会（改期）",
  "start_time": "2026-04-08T09:00:00+08:00",
  "end_time": "2026-04-08T10:00:00+08:00",
  "status": "active"
}
```

`status` 可选值：`active` / `cancelled` / `expired`。

**响应：** 更新后的日程对象。

#### 2.5 删除日程

```
DELETE /api/events/:id
```

**响应：** HTTP 204，无返回体。

---

### 3. 用户管理（仅管理员）

以下接口需要管理员权限。

#### 3.1 创建用户

```
POST /api/users
```

**请求体：**
```json
{
  "username": "zhangsan",
  "password": "zhangsan-password",
  "is_admin": false
}
```

**响应（201）：**
```json
{
  "user": {
    "id": "usr_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
    "username": "zhangsan",
    "is_admin": false,
    "created_at": "2026-04-04T15:00:00+00:00"
  },
  "api_key": "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
}
```

> **重要：** `api_key` 仅在创建时返回一次，请立即保存。

#### 3.2 查询用户列表

```
GET /api/users
```

**响应：**
```json
[
  {
    "id": "usr_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
    "username": "admin",
    "is_admin": true,
    "created_at": "2026-04-04T15:00:00+00:00"
  }
]
```

#### 3.3 获取单个用户

```
GET /api/users/:id
```

#### 3.4 更新用户

```
PUT /api/users/:id
```

**请求体：**
```json
{
  "username": "new-username"
}
```

#### 3.5 删除用户

```
DELETE /api/users/:id
```

**响应：** HTTP 204，无返回体。

---

### 4. Webhook 管理

#### 4.1 创建 Webhook

```
POST /api/webhooks
```

**请求体：**
```json
{
  "user_id": "usr_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "webhook": {
    "url": "https://your-server.com/callback",
    "events": ["event.created", "event.updated", "event.deleted"],
    "secret": "your-signing-secret"
  }
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `url` | string | 是 | 回调 URL，必须以 `http://` 或 `https://` 开头 |
| `events` | string[] | 是 | 监听的事件类型 |
| `secret` | string | 否 | 签名密钥 |

可用事件类型：

| 事件 | 触发时机 |
|------|---------|
| `event.created` | 创建日程后 |
| `event.updated` | 更新日程后 |
| `event.deleted` | 删除日程后 |

**响应（201）：**
```json
{
  "id": "wh_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "user_id": "usr_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "url": "https://your-server.com/callback",
  "events": ["event.created", "event.updated", "event.deleted"],
  "is_active": true,
  "created_at": "2026-04-04T15:00:00+00:00"
}
```

#### 4.2 查询 Webhook 列表

```
GET /api/webhooks
```

返回当前用户的所有 Webhook。

#### 4.3 获取单个 Webhook

```
GET /api/webhooks/:id
```

#### 4.4 更新 Webhook

```
PUT /api/webhooks/:id
```

**请求体（所有字段可选）：**
```json
{
  "url": "https://new-url.com/callback",
  "events": ["event.created"],
  "is_active": false
}
```

#### 4.5 删除 Webhook

```
DELETE /api/webhooks/:id
```

**响应：** HTTP 204，无返回体。

---

### 5. 日历订阅（ICS）

无需认证，可直接订阅到系统日历。

```
GET /calendar/:user_id/subscribe.ics
```

将此 URL 添加到 iPhone / macOS 日历即可同步查看日程。

**添加方式：**
1. 打开系统「设置」>「日历」>「账户」>「添加日历订阅账户」
2. 粘贴上述 URL
3. 保存即可

---

## 调用示例

### cURL

**创建日程：**
```bash
curl -X POST https://ics.xiuyuan.xin/api/events \
  -H "X-API-Key: your-api-key" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "usr_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
    "event": {
      "title": "团队会议",
      "start_time": "2026-04-07T14:00:00+08:00",
      "end_time": "2026-04-07T15:00:00+08:00"
    }
  }'
```

**查询日程：**
```bash
curl https://ics.xiuyuan.xin/api/events?status=active \
  -H "X-API-Key: your-api-key"
```

### Python

```python
import requests

BASE = "https://ics.xiuyuan.xin"
HEADERS = {"X-API-Key": "your-api-key"}

# 创建日程
resp = requests.post(f"{BASE}/api/events", headers={**HEADERS, "Content-Type": "application/json"}, json={
    "user_id": "usr_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
    "event": {
        "title": "团队会议",
        "start_time": "2026-04-07T14:00:00+08:00",
        "end_time": "2026-04-07T15:00:00+08:00",
        "tags": ["工作"]
    }
})
print(resp.json())

# 查询日程
resp = requests.get(f"{BASE}/api/events", headers=HEADERS, params={"status": "active"})
for event in resp.json():
    print(f"{event['title']}  {event['start_time']} ~ {event['end_time']}")

# 删除日程
requests.delete(f"{BASE}/api/events/{event_id}", headers=HEADERS)
```

### JavaScript

```javascript
const BASE = "https://ics.xiuyuan.xin";
const HEADERS = { "X-API-Key": "your-api-key", "Content-Type": "application/json" };

// 创建日程
const resp = await fetch(`${BASE}/api/events`, {
  method: "POST",
  headers: HEADERS,
  body: JSON.stringify({
    user_id: "usr_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
    event: {
      title: "团队会议",
      start_time: "2026-04-07T14:00:00+08:00",
      end_time: "2026-04-07T15:00:00+08:00"
    }
  })
});
const event = await resp.json();
console.log(event);

// 查询日程
const events = await fetch(`${BASE}/api/events?status=active`, { headers: HEADERS }).then(r => r.json());
```

---

## 权限说明

| 角色 | 创建日程 | 管理自己的日程 | 管理所有日程 | 用户管理 |
|------|---------|--------------|-------------|---------|
| Admin | 可以 | 可以 | 可以 | 可以 |
| User | 可以 | 可以（仅自己） | 不可以 | 不可以 |

- 普通用户只能操作自己的日程
- 管理员可以操作所有用户的日程
- Webhook 只能查看和操作自己创建的
