//! Testing utilities for integration tests
//!
//! This module provides a centralized test database setup using SQLite in-memory databases.
//! Each test gets an isolated database instance with the full schema applied.
//!
//! ## Usage
//!
//! ```rust,ignore
//! #[tokio::test]
//! async fn my_test() -> Result<()> {
//!     let db = TestDatabase::new().await?;
//!     // Use db.pool() for database operations
//!     Ok(())
//! }
//! ```

use anyhow::Result;
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_DB_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A test database with an in-memory SQLite instance
///
/// Each instance is isolated with a unique name and includes the full schema.
/// Perfect for integration tests that need a database without external setup.
#[derive(Clone)]
pub struct TestDatabase {
    pool: Pool<Sqlite>,
    db_id: usize,
}

impl TestDatabase {
    /// Create a new isolated test database with full schema
    pub async fn new() -> Result<Self> {
        let db_id = TEST_DB_COUNTER.fetch_add(1, Ordering::SeqCst);
        let db_name = format!("file:test_db_{}?mode=memory&cache=shared", db_id);

        let pool = SqlitePoolOptions::new().max_connections(1).connect(&db_name).await?;

        // Create the schema
        setup_schema(&pool).await?;

        Ok(Self { pool, db_id })
    }

    /// Get the connection pool for this database
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Reset the database by clearing all tables (TRUNCATE all data)
    ///
    /// This is faster than creating a new database and preserves the schema.
    /// Note: Be careful with foreign key constraints when truncating.
    pub async fn reset(&self) -> Result<()> {
        let mut conn = self.pool.acquire().await?;

        // Disable foreign key constraints temporarily
        sqlx::query("PRAGMA foreign_keys = OFF;").execute(&mut *conn).await?;

        // Delete all data from tables in reverse dependency order
        sqlx::query("DELETE FROM repositories;").execute(&mut *conn).await?;

        sqlx::query("DELETE FROM users;").execute(&mut *conn).await?;

        // Re-enable foreign key constraints
        sqlx::query("PRAGMA foreign_keys = ON;").execute(&mut *conn).await?;

        Ok(())
    }

    /// Get the database ID for debugging
    pub fn id(&self) -> usize {
        self.db_id
    }

    /// Check database health
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(self.pool()).await?;
        Ok(())
    }
}

/// Set up the test database schema (SQLite compatible)
///
/// This creates the same schema as the production migrations but in SQLite syntax.
/// It's a snapshot of the final schema state to avoid duplication of migration files.
async fn setup_schema(pool: &Pool<Sqlite>) -> Result<()> {
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
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            last_login DATETIME,
            last_activity DATETIME,
            avatar_url TEXT,
            bio TEXT,
            full_name TEXT,
            phone TEXT,
            timezone TEXT,
            preferences TEXT,
            login_count INTEGER NOT NULL DEFAULT 0
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
            last_crawl_error TEXT,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            auto_crawl_enabled BOOLEAN NOT NULL DEFAULT false,
            cron_schedule TEXT,
            next_crawl_at DATETIME,
            crawl_frequency_hours INTEGER,
            max_crawl_duration_minutes INTEGER,
            last_crawl_duration_seconds INTEGER,
            gitlab_excluded_projects TEXT,
            gitlab_excluded_patterns TEXT,
            github_namespace TEXT,
            github_excluded_repositories TEXT,
            github_excluded_patterns TEXT,
            crawl_state TEXT,
            last_processed_project TEXT,
            crawl_started_at DATETIME,
            included_branches TEXT,
            included_branches_patterns TEXT,
            excluded_branches TEXT,
            excluded_branches_patterns TEXT,
            included_projects TEXT,
            included_projects_patterns TEXT
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
        let db1 = TestDatabase::new().await.unwrap();
        let db2 = TestDatabase::new().await.unwrap();

        // Verify they have different IDs
        assert_ne!(db1.id(), db2.id());

        // Insert data in db1
        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash) VALUES ('1', 'user1', 'user1@test.com', 'hash1')",
        )
        .execute(db1.pool())
        .await
        .unwrap();

        // Insert different data in db2
        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash) VALUES ('2', 'user2', 'user2@test.com', 'hash2')",
        )
        .execute(db2.pool())
        .await
        .unwrap();

        // Verify isolation
        let count1: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(db1.pool()).await.unwrap();

        let count2: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(db2.pool()).await.unwrap();

        assert_eq!(count1, 1);
        assert_eq!(count2, 1);
    }

    #[tokio::test]
    async fn test_database_reset() {
        let db = TestDatabase::new().await.unwrap();

        // Insert data
        sqlx::query(
            "INSERT INTO users (id, username, email, password_hash) VALUES ('1', 'user1', 'user1@test.com', 'hash1')",
        )
        .execute(db.pool())
        .await
        .unwrap();

        let count_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(db.pool()).await.unwrap();

        assert_eq!(count_before, 1);

        // Reset
        db.reset().await.unwrap();

        // Verify data is cleared but schema still exists
        let count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(db.pool()).await.unwrap();

        assert_eq!(count_after, 0);
    }

    #[tokio::test]
    async fn test_health_check() {
        let db = TestDatabase::new().await.unwrap();
        assert!(db.health_check().await.is_ok());
    }
}
