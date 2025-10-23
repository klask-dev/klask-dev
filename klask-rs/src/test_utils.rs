use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_DB_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// Create an isolated in-memory SQLite database for testing
pub async fn create_test_database() -> Result<Pool<Sqlite>> {
    let counter = TEST_DB_COUNTER.fetch_add(1, Ordering::SeqCst);
    let db_name = format!("file:test_db_{}?mode=memory&cache=shared", counter);

    let pool = SqlitePoolOptions::new()
        .max_connections(1) // SQLite in-memory works best with single connection
        .connect(&db_name)
        .await?;

    // Run migrations - we'll need to adapt them for SQLite
    // For now, create minimal schema manually
    setup_test_schema(&pool).await?;

    Ok(pool)
}

async fn setup_test_schema(pool: &Pool<Sqlite>) -> Result<()> {
    // Create tables compatible with both PostgreSQL and SQLite
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id TEXT PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            email TEXT UNIQUE NOT NULL,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL DEFAULT 'User',
            active BOOLEAN NOT NULL DEFAULT true,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS repositories (
            id TEXT PRIMARY KEY,
            name TEXT UNIQUE NOT NULL,
            url TEXT NOT NULL,
            repository_type TEXT NOT NULL,
            branch TEXT,
            enabled BOOLEAN NOT NULL DEFAULT true,
            access_token TEXT,
            gitlab_namespace TEXT,
            is_group BOOLEAN NOT NULL DEFAULT false,
            last_crawled DATETIME,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            auto_crawl_enabled BOOLEAN NOT NULL DEFAULT false,
            cron_schedule TEXT,
            next_crawl_at DATETIME,
            crawl_frequency_hours INTEGER,
            max_crawl_duration_minutes INTEGER
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_isolated_databases() {
        let db1 = create_test_database().await.unwrap();
        let db2 = create_test_database().await.unwrap();

        // Insert data in db1
        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash) VALUES ('1', 'user1', 'user1@test.com', 'hash1')",
        )
        .execute(&db1)
        .await
        .unwrap();

        // Insert different data in db2
        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash) VALUES ('2', 'user2', 'user2@test.com', 'hash2')",
        )
        .execute(&db2)
        .await
        .unwrap();

        // Verify isolation
        let count1: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(&db1).await.unwrap();

        let count2: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(&db2).await.unwrap();

        assert_eq!(count1, 1);
        assert_eq!(count2, 1);
    }
}
