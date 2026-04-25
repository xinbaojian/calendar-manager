# CalendarSync MCP 使用文档

CalendarSync 内置了 MCP (Model Context Protocol) 服务器，允许 AI 助手（如 Claude、ChatGPT）通过标准化协议管理日历日程。

## 快速开始

### 端点地址

```
POST https://ics.xiuyuan.xin/mcp
```

### 认证方式

所有 MCP 请求需要通过 `X-API-Key` 请求头进行认证：

```
X-API-Key: <your_api_key>
```

API Key 可通过 Web 管理界面（设置页）或 REST API（`POST /api/auth/api-key`）获取。

> MCP 仅支持 API Key 认证，不支持 JWT Token。

### 基本请求格式

MCP 使用 JSON-RPC 2.0 协议：

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "<工具名称>",
    "arguments": { ... }
  }
}
```

### curl 示例

```bash
curl -X POST https://ics.xiuyuan.xin/mcp \
  -H "Content-Type: application/json" \
  -H "X-API-Key: your-api-key" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "tools/call",
    "params": {
      "name": "list_events",
      "arguments": { "status": "active" }
    }
  }'
```

---

## 工具列表

### 1. create_event — 创建日程

创建一个新的日历日程事件。

**参数：**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `title` | string | 是 | 日程标题，不能为空 |
| `description` | string | 否 | 日程描述 |
| `location` | string | 否 | 地点 |
| `start_time` | string | 是 | 开始时间，RFC3339 格式 |
| `end_time` | string | 是 | 结束时间，RFC3339 格式 |
| `recurrence_rule` | string | 否 | 重复规则，RRULE 格式 |
| `recurrence_until` | string | 否 | 重复结束时间，RFC3339 格式 |
| `reminder_minutes` | integer | 否 | 提前提醒分钟数 |
| `tags` | string[] | 否 | 标签列表 |

**时间格式说明：** 所有时间使用 RFC3339 格式，默认时区为 `Asia/Shanghai (UTC+8)`。

```
2026-05-01T10:00:00+08:00
```

**RRULE 格式示例：**

| 规则 | 说明 |
|------|------|
| `FREQ=DAILY` | 每天重复 |
| `FREQ=WEEKLY` | 每周重复 |
| `FREQ=MONTHLY` | 每月重复 |
| `FREQ=WEEKLY;BYDAY=MO,WE,FR` | 每周一、三、五 |
| `FREQ=DAILY;INTERVAL=2` | 每隔 2 天 |
| `FREQ=DAILY;COUNT=10` | 重复 10 次 |
| `FREQ=DAILY;UNTIL=20261231T235959Z` | 重复到指定日期 |

**请求示例：**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "create_event",
    "arguments": {
      "title": "团队周会",
      "description": "讨论本周工作进展和下周计划",
      "location": "会议室 302",
      "start_time": "2026-05-01T14:00:00+08:00",
      "end_time": "2026-05-01T15:30:00+08:00",
      "reminder_minutes": 15,
      "tags": ["工作", "会议"]
    }
  }
}
```

**创建重复日程：**

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "tools/call",
  "params": {
    "name": "create_event",
    "arguments": {
      "title": "每日站会",
      "start_time": "2026-05-01T09:00:00+08:00",
      "end_time": "2026-05-01T09:15:00+08:00",
      "recurrence_rule": "FREQ=WEEKLY;BYDAY=MO,TU,WE,TH,FR",
      "recurrence_until": "2026-06-30T23:59:59+08:00"
    }
  }
}
```

**返回值：** 成功时返回日程 ID（如 `"evt_550e8400-e29b-41d4-a716-446655440000"`）。

---

### 2. list_events — 查询日程列表

根据条件查询日程列表，所有参数均为可选。

**参数：**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `from` | string | 否 | 开始日期过滤，RFC3339 格式 |
| `to` | string | 否 | 结束日期过滤，RFC3339 格式 |
| `status` | string | 否 | 状态过滤：`active`、`expired`、`all` |
| `keyword` | string | 否 | 关键词搜索（匹配标题和描述） |

**请求示例：**

查询本月所有活跃日程：

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/call",
  "params": {
    "name": "list_events",
    "arguments": {
      "from": "2026-05-01T00:00:00+08:00",
      "to": "2026-05-31T23:59:59+08:00",
      "status": "active"
    }
  }
}
```

按关键词搜索：

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "list_events",
    "arguments": {
      "keyword": "会议"
    }
  }
}
```

**返回值：** 日程 JSON 数组，每个元素包含日程完整信息。

---

### 3. get_event — 获取日程详情

根据 ID 获取单个日程的详细信息。

**参数：**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | string | 是 | 日程 ID，不能为空 |

**请求示例：**

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "tools/call",
  "params": {
    "name": "get_event",
    "arguments": {
      "id": "evt_550e8400-e29b-41d4-a716-446655440000"
    }
  }
}
```

**返回值：** 日程 JSON 对象，包含所有字段。

---

### 4. update_event — 更新日程

更新现有日程，仅需传入要修改的字段。

**参数：**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | string | 是 | 日程 ID |
| `title` | string | 否 | 新标题 |
| `description` | string | 否 | 新描述 |
| `location` | string | 否 | 新地点 |
| `start_time` | string | 否 | 新开始时间，RFC3339 格式 |
| `end_time` | string | 否 | 新结束时间，RFC3339 格式 |
| `status` | string | 否 | 状态：`active`、`expired` |
| `recurrence_rule` | string | 否 | 重复规则，RRULE 格式 |
| `recurrence_until` | string | 否 | 重复结束时间 |
| `reminder_minutes` | integer | 否 | 提前提醒分钟数 |
| `tags` | string[] | 否 | 标签列表 |

**请求示例：**

修改标题和地点：

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "tools/call",
  "params": {
    "name": "update_event",
    "arguments": {
      "id": "evt_550e8400-e29b-41d4-a716-446655440000",
      "title": "团队周会（改为线上）",
      "location": "腾讯会议"
    }
  }
}
```

> **注意：** 将过期日程的结束时间改为未来时间时，状态会自动恢复为 `active`。

**返回值：** 更新后的日程 JSON 对象。

---

### 5. delete_event — 删除日程

删除指定日程（软删除，状态变为 `cancelled`）。

**参数：**

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `id` | string | 是 | 日程 ID |

**请求示例：**

```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "tools/call",
  "params": {
    "name": "delete_event",
    "arguments": {
      "id": "evt_550e8400-e29b-41d4-a716-446655440000"
    }
  }
}
```

**返回值：** 成功时返回 `"删除成功"`。

---

## 错误处理

### 错误响应格式

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32602,
    "message": "Invalid params: 日程标题不能为空"
  }
}
```

### 错误码说明

| 错误码 | 类型 | 说明 |
|--------|------|------|
| `-32602` | Invalid Params | 参数验证失败（必填字段为空、时间格式错误、状态值无效等） |
| `-32001` | Resource Not Found | 日程不存在 |
| `-32603` | Internal Error | 服务器内部错误（数据库操作失败、序列化错误等） |

### 常见错误场景

| 场景 | 错误码 | 错误信息示例 |
|------|--------|------------|
| 缺少 API Key | HTTP 401 | `API Key 无效` |
| API Key 无效 | HTTP 401 | `API Key 无效` |
| 标题为空 | `-32602` | `日程标题不能为空` |
| 时间格式错误 | `-32602` | `无效的开始时间格式: 2026-05-01` |
| 日程不存在 | `-32001` | `日程不存在: evt_xxx` |
| 无权访问他人日程 | `-32603` | `日程不存在或无权访问` |
| 无效状态值 | `-32602` | `无效的状态值: invalid，可选值: active, expired, all` |

---

## 权限说明

- 每个请求通过 API Key 关联到具体用户
- 普通用户只能操作自己创建的日程
- 管理员用户拥有完整访问权限
- 每个请求独立认证，无会话状态

---

## Webhook 集成

MCP 操作与 REST API 共享相同的 webhook 通知机制：

| 操作 | 事件类型 |
|------|---------|
| 创建日程 | `event.created` |
| 更新日程 | `event.updated` |
| 删除日程 | `event.deleted` |

Webhook 通知为异步发送，不影响 MCP 响应速度。通知失败不会导致操作回滚。

---

## AI 助手配置示例

### Claude Desktop

在 `claude_desktop_config.json` 中添加：

```json
{
  "mcpServers": {
    "calendar": {
      "url": "https://ics.xiuyuan.xin/mcp",
      "headers": {
        "X-API-Key": "your_api_key"
      }
    }
  }
}
```

### 通用 HTTP MCP 客户端

任何支持 HTTP 传输的 MCP 客户端都可以通过以下配置连接：

- **端点：** `http://<host>:<port>/mcp`
- **认证头：** `X-API-Key: <api-key>`
- **协议：** JSON-RPC 2.0
- **传输：** HTTP (Streamable HTTP)
