use chrono::Utc;
use chrono_tz::Asia::Shanghai;
use sqlx::{Pool, Sqlite};

use crate::error::{AppError, AppResult};
use crate::models::{CreateEvent, Event, QueryEvents, UpdateEvent};

#[derive(Clone)]
pub struct EventRepository {
    pool: Pool<Sqlite>,
}

impl EventRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn create(&self, user_id: String, input: CreateEvent) -> AppResult<Event> {
        let event = Event::new(user_id, input)
            .map_err(AppError::ValidationError)?;

        sqlx::query(
            "INSERT INTO events (id, user_id, title, description, location, start_time, end_time,
             recurrence_rule, recurrence_until, reminder_minutes, tags, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
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

    #[tracing::instrument(skip(self), fields(event_id = %id))]
    pub async fn find_by_id(&self, id: &str) -> AppResult<Event> {
        sqlx::query_as::<_, Event>("SELECT * FROM events WHERE id = ?1")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|_| AppError::EventNotFound(id.to_string()))
    }

    #[tracing::instrument(skip(self), fields(user_id = %user_id))]
    pub async fn find_by_user(&self, user_id: &str, query: QueryEvents) -> AppResult<Vec<Event>> {
        let mut sql = "SELECT * FROM events WHERE user_id = ?1".to_string();
        let mut bind_values: Vec<String> = vec![user_id.to_string()];
        let mut bind_idx = 1;

        if let Some(status) = &query.status {
            bind_idx += 1;
            sql.push_str(&format!(" AND status = ?{bind_idx}"));
            bind_values.push(status.clone());
        }

        if let Some(from) = &query.from {
            bind_idx += 1;
            sql.push_str(&format!(" AND start_time >= ?{bind_idx}"));
            bind_values.push(from.clone());
        }

        if let Some(to) = &query.to {
            bind_idx += 1;
            sql.push_str(&format!(" AND start_time <= ?{bind_idx}"));
            bind_values.push(to.clone());
        }

        if let Some(keyword) = &query.keyword {
            bind_idx += 1;
            let next_idx = bind_idx + 1;
            sql.push_str(&format!(
                " AND (title LIKE ?{bind_idx} ESCAPE '\\' OR description LIKE ?{next_idx} ESCAPE '\\')",
            ));
            // Escape LIKE wildcards by prepending backslash
            let escaped = keyword.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
            let pattern = format!("%{escaped}%");
            bind_values.push(pattern.clone());
            bind_values.push(pattern);
        }

        sql.push_str(" ORDER BY start_time ASC");

        let mut query_builder = sqlx::query_as::<_, Event>(&sql);
        for val in bind_values {
            query_builder = query_builder.bind(val);
        }

        query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(AppError::Database)
    }

    pub async fn update(&self, id: &str, input: UpdateEvent) -> AppResult<Event> {
        input.validate()
            .map_err(AppError::ValidationError)?;

        let mut event = self.find_by_id(id).await?;
        let now = Utc::now().with_timezone(&Shanghai).to_rfc3339();

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
            event.tags = Some(serde_json::to_string(&tags).map_err(AppError::Serialization)?);
        }
        if let Some(status) = input.status {
            event.status = status;
        }

        sqlx::query(
            "UPDATE events SET title = ?1, description = ?2, location = ?3, start_time = ?4,
             end_time = ?5, recurrence_rule = ?6, recurrence_until = ?7, reminder_minutes = ?8,
             tags = ?9, status = ?10, updated_at = ?11 WHERE id = ?12",
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
        sqlx::query("UPDATE events SET status = 'cancelled', updated_at = ?1 WHERE id = ?2 AND status != 'cancelled'")
            .bind(Utc::now().with_timezone(&Shanghai).to_rfc3339())
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(AppError::Database)?;

        Ok(())
    }

    pub async fn mark_expired(&self, before: &str) -> AppResult<u64> {
        let result = sqlx::query(
            "UPDATE events SET status = 'expired', updated_at = ?1
             WHERE status = 'active' AND end_time < ?2",
        )
        .bind(Utc::now().with_timezone(&Shanghai).to_rfc3339())
        .bind(before)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        Ok(result.rows_affected())
    }

    pub async fn delete_old_expired(&self, days: i64) -> AppResult<u64> {
        let cutoff = Utc::now().with_timezone(&Shanghai) - chrono::Duration::days(days);

        let result =
            sqlx::query("DELETE FROM events WHERE status = 'expired' AND updated_at < ?1")
                .bind(cutoff.to_rfc3339())
                .execute(&self.pool)
                .await
                .map_err(AppError::Database)?;

        Ok(result.rows_affected())
    }
}
