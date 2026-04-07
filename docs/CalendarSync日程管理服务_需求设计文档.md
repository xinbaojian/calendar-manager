# 日程管理微服务 - 需求设计文档

**版本**：v1.0
**日期**：2026-04-04
**作者**：OpenClaw AI

---

## 一、项目概述

### 1.1 项目名称
**CalendarSync** - 轻量级日程同步服务

### 1.2 项目目标
提供一个轻量级的日程管理服务，支持：
- iPhone 日历订阅（.ics 格式）
- API 增删改查日程
- 自动清理过期日程
- 历史日程查询

### 1.3 技术选型
| 项目 | 选择 | 理由 |
|------|------|------|
| 语言 | **Rust** | 内存占用极低、性能高、适合长期运行的服务 |
| Web框架 | Axum | 异步高性能、生态成熟 |
| 数据库 | SQLite | 轻量级、单文件、无需额外部署 |
| 日历格式 | iCalendar (RFC 5545) | 标准格式，iPhone 原生支持 |
| 部署 | Docker | 跨平台、易部署 |

### 1.4 资源预估
- **内存占用**：< 20MB
- **磁盘占用**：< 50MB（含数据库）
- **CPU**：极低，空闲时几乎为 0

---

## 二、功能需求

### 2.1 日程管理 API

#### 2.1.1 创建日程
```
POST /api/events
```

**请求体**：
```json
{
  "title": "找出爸爸的身份证",
  "description": "身份证在主卧衣柜抽屉里放着，去医院用",
  "location": "主卧衣柜抽屉",
  "start_time": "2026-04-09T19:00:00+08:00",
  "end_time": "2026-04-09T20:00:00+08:00",
  "reminder_minutes": 60,
  "tags": ["家庭", "重要"]
}
```

**响应**：
```json
{
  "id": "evt_abc123",
  "created_at": "2026-04-04T00:47:00+08:00"
}
```

#### 2.1.2 查询日程列表
```
GET /api/events?status=active&from=2026-04-01&to=2026-04-30
```

**参数**：
- `status`: `active`（未过期）| `all`（全部）| `expired`（已过期）
- `from`: 开始日期（可选）
- `to`: 结束日期（可选）
- `keyword`: 搜索关键词（可选）

**响应**：
```json
{
  "events": [
    {
      "id": "evt_abc123",
      "title": "找出爸爸的身份证",
      "description": "身份证在主卧衣柜抽屉里放着",
      "start_time": "2026-04-09T19:00:00+08:00",
      "end_time": "2026-04-09T20:00:00+08:00",
      "status": "active"
    }
  ],
  "total": 1
}
```

#### 2.1.3 查询单个日程
```
GET /api/events/{id}
```

#### 2.1.4 更新日程
```
PUT /api/events/{id}
```

**请求体**：同创建日程

#### 2.1.5 删除日程
```
DELETE /api/events/{id}
```

**响应**：
```json
{
  "success": true,
  "message": "Event deleted"
}
```

#### 2.1.6 搜索历史日程
```
GET /api/events/search?keyword=身份证&include_expired=true
```

---

### 2.2 日历订阅接口

#### 2.2.1 订阅链接
```
GET /calendar/subscribe.ics
```

**说明**：
- 返回标准 iCalendar 格式文件
- 仅返回**未过期**的日程
- 支持在 iPhone 日历 App 中添加订阅

**响应头**：
```
Content-Type: text/calendar; charset=utf-8
Cache-Control: no-cache
```

**响应体示例**：
```ics
BEGIN:VCALENDAR
VERSION:2.0
PRODID:-//CalendarSync//CN
CALSCALE:GREGORIAN
METHOD:PUBLISH
X-WR-CALNAME:我的日程
X-WR-CALDESC:CalendarSync 日程订阅
BEGIN:VEVENT
DTSTART:20260409T190000
DTEND:20260409T200000
DTSTAMP:20260403T164700Z
UID:evt_abc123@calendarsync
SUMMARY:找出爸爸的身份证
DESCRIPTION:身份证在主卧衣柜抽屉里放着\n去医院用
LOCATION:主卧衣柜抽屉
STATUS:CONFIRMED
BEGIN:VALARM
TRIGGER:-PT60M
ACTION:DISPLAY
DESCRIPTION:日程提醒
END:VALARM
END:VEVENT
END:VCALENDAR
```

#### 2.2.2 iPhone 订阅方式
1. 设置 → 日历 → 账户 → 添加账户 → 其他
2. 添加已订阅的日历
3. 输入 URL：`https://your-server.com/calendar/subscribe.ics`
4. 保存，自动同步

---

### 2.3 定时清理任务

#### 2.3.1 逻辑删除机制
- 日程过期后，自动标记为 `expired` 状态
- 过期日程**不在订阅接口返回**
- 但可通过搜索 API 查询历史

#### 2.3.2 清理规则
| 规则 | 说明 |
|------|------|
| 检查频率 | 每小时检查一次 |
| 过期判定 | `end_time < 当前时间` |
| 状态变更 | `active` → `expired` |
| 物理删除 | 不进行物理删除，保留历史记录 |

#### 2.3.3 可选：自动物理删除
```
# 配置项
AUTO_DELETE_EXPIRED_DAYS=30  # 过期30天后物理删除，0表示不删除
```

---

## 三、数据模型

### 3.1 日程表 (events)
```sql
CREATE TABLE events (
    id TEXT PRIMARY KEY,           -- evt_xxx 格式
    title TEXT NOT NULL,           -- 日程标题
    description TEXT,              -- 日程描述
    location TEXT,                 -- 地点
    start_time TEXT NOT NULL,      -- 开始时间 (ISO 8601)
    end_time TEXT NOT NULL,        -- 结束时间 (ISO 8601)
    reminder_minutes INTEGER,      -- 提前提醒分钟数
    tags TEXT,                     -- JSON 数组
    status TEXT DEFAULT 'active',  -- active/expired/deleted
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_start_time ON events(start_time);
CREATE INDEX idx_status ON events(status);
```

---

## 四、API 认证（已确认方案）

### 4.1 认证策略
| 接口类型 | 认证要求 | 说明 |
|---------|---------|------|
| 日历订阅 `GET /calendar/subscribe.ics` | **公开** | 只有查询权限，日程无隐私敏感 |
| 查询日程 `GET /api/events` | 需要 API Key | 防止接口被滥用 |
| 创建日程 `POST /api/events` | 需要 API Key | 写操作必须认证 |
| 更新日程 `PUT /api/events/{id}` | 需要 API Key | 写操作必须认证 |
| 删除日程 `DELETE /api/events/{id}` | 需要 API Key | 写操作必须认证 |

### 4.2 认证方式：API Key + HTTPS
```
# 请求头携带 API Key
X-API-Key: your-secret-key-xxx

# 示例
curl -H "X-API-Key: xxx" https://your-server.com/api/events
```

### 4.3 HTTPS 部署（必须）
```
部署架构：
公网 → Nginx(HTTPS:443) → CalendarSync:8080(HTTP)
              ↓
      SSL 终结 + 反向代理
      Let's Encrypt 免费证书
```

**Nginx 配置示例**：
```nginx
server {
    listen 443 ssl;
    server_name calendar.yourdomain.com;

    ssl_certificate /etc/letsencrypt/live/yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/yourdomain.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}

# HTTP 自动跳转 HTTPS
server {
    listen 80;
    server_name calendar.yourdomain.com;
    return 301 https://$server_name$request_uri;
}
```

### 4.4 配置文件
```toml
# config.toml
[server]
host = "127.0.0.1"  # 仅监听本地，通过 Nginx 暴露
port = 8080

[auth]
api_key = "your-secret-key-change-me"  # 生产环境请修改

[database]
path = "./data/calendar.db"

[cleanup]
check_interval_hours = 1
```

### 4.5 安全总结
- ✅ HTTPS 加密传输（Nginx + Let's Encrypt）
- ✅ 写操作需要 API Key 认证
- ✅ 订阅链接公开（仅读，无敏感信息）
- ✅ 服务仅监听本地 127.0.0.1

---

## 五、Docker 部署

### 5.1 Dockerfile
```dockerfile
FROM rust:1.75-alpine AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine:3.19
RUN apk add --no-cache ca-certificates tzdata
COPY --from=builder /app/target/release/calendarsync /usr/local/bin/
EXPOSE 8080
CMD ["calendarsync"]
```

### 5.2 docker-compose.yml
```yaml
version: '3.8'
services:
  calendarsync:
    image: calendarsync:latest
    container_name: calendarsync
    restart: unless-stopped
    ports:
      - "8080:8080"
    volumes:
      - ./data:/app/data
      - ./config.toml:/app/config.toml:ro
    environment:
      - TZ=Asia/Shanghai
    mem_limit: 50m
```

---

## 六、项目结构

```
calendarsync/
├── Cargo.toml
├── src/
│   ├── main.rs              # 入口
│   ├── config.rs            # 配置加载
│   ├── db/
│   │   ├── mod.rs
│   │   └── schema.sql       # 数据库初始化
│   ├── handlers/
│   │   ├── events.rs        # 日程 CRUD API
│   │   └── calendar.rs      # 订阅接口
│   ├── models/
│   │   └── event.rs         # 数据模型
│   ├── scheduler/
│   │   └── cleanup.rs       # 定时清理任务
│   └── ical/
│       └── generator.rs     # iCalendar 生成器
├── config.toml              # 配置文件
├── Dockerfile
└── docker-compose.yml
```

---

## 七、依赖库

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
axum = "0.7"
tower = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.7", features = ["runtime-tokio", "sqlite"] }
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
toml = "0.8"
tracing = "0.1"
tracing-subscriber = "0.3"
```

---

## 八、开发计划

| 阶段 | 任务 | 预计时间 |
|------|------|----------|
| Phase 1 | 项目初始化 + 数据库模型 | 1天 |
| Phase 2 | CRUD API 开发 | 1天 |
| Phase 3 | 日历订阅接口 | 0.5天 |
| Phase 4 | 定时清理任务 | 0.5天 |
| Phase 5 | Docker 部署配置 | 0.5天 |
| Phase 6 | 测试 + 文档 | 1天 |

**总计**：约 4-5 天

---

## 九、部署方案（已确认：Docker）

### 9.1 选择理由
- 服务器已有 Docker 环境，跑其他服务
- 统一管理，迁移便利
- 内存开销可接受（几十 MB 换运维便利）

### 9.2 部署步骤
```bash
# 1. 构建镜像
docker build -t calendarsync:latest .

# 2. 启动服务
docker-compose up -d

# 3. 查看日志
docker logs -f calendarsync
```

### 9.3 资源预估（Docker 环境）
| 项目 | 占用 |
|------|------|
| Rust 程序 | ~15MB |
| Alpine 基础镜像 | ~5MB |
| 容器运行时开销 | ~5MB |
| **合计** | ~25-30MB |

---

## 十、扩展功能（也实现一下）

### 9.1 多用户支持
- 用户注册/登录
- 每个用户独立的订阅链接

### 9.2 重复日程
- 支持每日/每周/每月重复

### 9.3 Webhook 通知
- 日程到期时触发 Webhook
- 可对接微信/钉钉/飞书

### 9.4 Web 管理界面
- 简单的日程管理页面
- 可视化日历视图

---

## 十一、总结

本项目是一个**极轻量级**的日程同步服务：
- 内存占用 < 20MB
- 单一可执行文件 + SQLite
- Docker 一键部署
- iPhone 日历原生支持
- API 简单易用

适合个人使用，可作为 OpenClaw 的日程后端服务。

---

**文档版本**：v1.0
**最后更新**：2026-04-04