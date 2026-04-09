# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CalendarSync is a lightweight calendar management service written in Rust that provides:
- Multi-user support with API key and JWT authentication
- RESTful API for calendar events, users, and webhooks
- iPhone calendar subscription via `.ics` format
- Recurring events support
- Webhook notifications for event changes
- Web management interface

## Development Commands

### Running the Application

```bash
# Run with default config (config.toml)
cargo run

# Run with custom config path
CONFIG_PATH=/path/to/config.toml cargo run
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test module
cargo test api_tests
cargo test timezone_tests

# Run tests with output
cargo test -- --nocapture
```

### Docker Development

```bash
# Local build
make build

# Cross-platform build (linux/amd64)
make buildx

# Build and push to registry
make push

# Release with version tag
make release VERSION=1.0.0
```

## Architecture

### Layer Structure

The application follows a clean architecture with clear separation:

- **`src/handlers/`** - HTTP request handlers (Axum), authentication middleware
- **`src/db/repositories/`** - Database access layer using repository pattern
- **`src/models/`** - Domain models (User, Event, Webhook)
- **`src/services/`** - Business logic (webhook delivery)
- **`src/ical/`** - iCalendar (.ics) generation for calendar subscriptions
- **`src/error.rs`** - Centralized error handling with AppError enum
- **`src/state.rs`** - Application state with Arc-wrapped repositories

### Database

- **SQLite** with sqlx for async database operations
- Migrations in `migrations/` directory (run automatically on startup)
- All timestamps stored in **Asia/Shanghai timezone** (UTC+8)
- Foreign key cascade deletes enabled (users → events/webhooks)

### Authentication

Two authentication methods:
1. **API Key** - Header `X-API-Key` for programmatic access
2. **JWT Token** - Password-based login with JWT tokens for web UI

Admin user is auto-created on first run with credentials from `config.toml`.

### Route Structure

- **Public routes**: `/`, `/api/auth/login`, `/calendar/:user_id/subscribe.ics`
- **Protected API routes**: `/api/*` (require auth_middleware)
- **SPA fallback**: `/events`, `/settings`, `/webhooks`, `/users` serve index.html

### Timezone Handling

**Critical**: All timestamps use **Asia/Shanghai (UTC+8)** timezone throughout the application:
- Database storage: RFC3339 strings with Shanghai timezone
- iCal generation: Explicit TZID=Asia/Shanghai
- Scheduled cleanup: Uses Shanghai timezone for expiration checks

Configuration via environment variables:
- `ADMIN_USERNAME` - Default: "admin"
- `ADMIN_API_KEY` - Default: "admin-secret-key-change-me" (MUST be changed)
- `JWT_SECRET` - Default: "change-this-jwt-secret-in-production"
- `CONFIG_PATH` - Custom config file path

### Webhook System

- Asynchronous delivery with retry (exponential backoff)
- HMAC-SHA256 signature support via `X-Webhook-Signature` header
- Webhook logs stored in `webhook_logs` table
- Configurable timeout and max retries

### Error Handling

All errors implement `IntoResponse` trait returning JSON:
```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable message",
    "details": {}  // optional
  }
}
```

### Background Tasks

Scheduled cleanup task runs every `check_interval_hours`:
1. Marks expired events as status='expired'
2. Deletes expired events older than `auto_delete_expired_days`

### Recurrence Rules

The application supports iCal-compatible recurrence rules (RRULE format) for recurring events:

- **Storage**: `recurrence_rule` field stores RRULE string in Event model
- **Generation**: Frontend `RecurrenceRuleGenerator` class creates RRULE from UI inputs
- **Export**: iCal generator includes RRULE in VEVENT output (RFC 5545 compliant)
- **Timezone**: All recurrence times use Asia/Shanghai (UTC+8)

**RRULE Format Examples:**
- `FREQ=DAILY` - Every day
- `FREQ=WEEKLY` - Every week
- `FREQ=MONTHLY` - Every month
- `FREQ=YEARLY` - Every year
- `FREQ=WEEKLY;BYDAY=MO,WE,FR` - Mon, Wed, Fri
- `FREQ=DAILY;INTERVAL=2` - Every 2 days
- `FREQ=WEEKLY;BYDAY=1MO` - First Monday of month
- `FREQ=DAILY;COUNT=10` - 10 occurrences
- `FREQ=DAILY;UNTIL=20251231T235959Z` - Until date

**Frontend Components:**
- RecurrenceEditor UI in event modal (`templates/index.html`)
- Preset options: none, daily, weekly, monthly, yearly, custom
- Custom settings: interval, frequency, day-of-week selection, end conditions

## Configuration

Edit `config.toml`:

```toml
[server]
host = "127.0.0.1"
port = 8080

[database]
path = "./data/calendar.db"

[auth]
admin_username = "admin"
admin_api_key = "MUST_CHANGE_THIS"
admin_password = "changeme"
jwt_secret = "MUST_CHANGE_THIS"
jwt_exp_hours = 24

[cleanup]
check_interval_hours = 1
auto_delete_expired_days = 30

[webhook]
timeout_seconds = 10
max_retries = 3
```

## Testing Strategy

Tests use in-memory SQLite databases (`/tmp/calendarsync-test-{uuid}.db`) with full migration runs. Each test gets isolated state.

## Important Notes

1. **Never commit config.toml** - contains secrets (use config.example.toml)
2. **Always use Shanghai timezone** when working with timestamps
3. **API key validation happens in middleware** - handlers receive AuthenticatedUser
4. **Repository methods return AppResult<T>** - centralized error handling
5. **iCal line folding** - lines > 75 octets folded per RFC 5545

## 交流建议

- 输出使用简体中文
