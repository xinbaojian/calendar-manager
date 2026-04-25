use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Row, Sqlite, SqlitePool};

pub async fn create_pool(database_url: &str) -> anyhow::Result<SqlitePool> {
    // 处理 sqlite:: 前缀，提取实际文件路径
    let file_path = database_url
        .strip_prefix("sqlite::")
        .unwrap_or(database_url);

    // 内存数据库不需要创建目录
    if file_path != ":memory:" {
        // 创建数据库文件的父目录
        if let Some(parent) = std::path::Path::new(file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // 使用 SqliteConnectOptions 以便自动创建数据库文件
    let opts = SqliteConnectOptions::new()
        .filename(file_path)
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new().connect_with(opts).await?;

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &Pool<Sqlite>) -> anyhow::Result<()> {
    let migration_sql = include_str!("../../migrations/001_initial_schema.sql");

    for statement in migration_sql.split(';') {
        let statement = statement.trim();
        if !statement.is_empty() {
            // 忽略已存在的表错误
            if let Err(e) = sqlx::query(statement).execute(pool).await {
                let err_msg = e.to_string();
                // SQLite 错误码 1: SQL 错误或缺少数据库（表已存在）
                if err_msg.contains("already exists") {
                    continue;
                }
                return Err(e.into());
            }
        }
    }

    // 为已有数据库添加 password_hash 列（若不存在）
    let result =
        sqlx::query("SELECT COUNT(*) FROM pragma_table_info('users') WHERE name = 'password_hash'")
            .fetch_one(pool)
            .await?;

    let count: i32 = result.try_get("COUNT(*)").unwrap_or(0);
    if count == 0 {
        sqlx::query("ALTER TABLE users ADD COLUMN password_hash TEXT")
            .execute(pool)
            .await?;
    }

    Ok(())
}
