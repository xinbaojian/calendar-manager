use sqlx::{Pool, Sqlite, SqlitePool};

pub async fn create_pool(database_url: &str) -> anyhow::Result<SqlitePool> {
    if let Some(parent) = std::path::Path::new(database_url).parent() {
        std::fs::create_dir_all(parent)?;
    }

    let pool = SqlitePool::connect(database_url).await?;

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await?;

    Ok(pool)
}

pub async fn run_migrations(pool: &Pool<Sqlite>) -> anyhow::Result<()> {
    let migration_sql = std::fs::read_to_string("migrations/001_initial_schema.sql")?;
    let mut tx = pool.begin().await?;

    for statement in migration_sql.split(';') {
        let statement = statement.trim();
        if !statement.is_empty() {
            sqlx::query(statement).execute(&mut *tx).await?;
        }
    }

    tx.commit().await?;
    Ok(())
}
