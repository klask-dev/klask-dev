use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

#[cfg(any(test, debug_assertions))]
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
#[cfg(any(test, debug_assertions))]
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn new(database_url: &str, max_connections: u32) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(Duration::from_secs(10))
            .connect(database_url)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }
}

// Test database using SQLite in-memory
#[cfg(any(test, debug_assertions))]
#[allow(dead_code)]
static TEST_DB_COUNTER: AtomicUsize = AtomicUsize::new(0);

#[cfg(any(test, debug_assertions))]
#[allow(dead_code)]
pub struct TestDatabase {
    pool: Pool<Sqlite>,
}

#[cfg(any(test, debug_assertions))]
impl TestDatabase {
    #[allow(dead_code)]
    pub async fn new() -> Result<Self> {
        let pool = create_test_database().await?;
        Ok(Self { pool })
    }

    #[allow(dead_code)]
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    #[allow(dead_code)]
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }
}

#[cfg(any(test, debug_assertions))]
#[allow(dead_code)]
pub async fn create_test_database() -> Result<Pool<Sqlite>> {
    let counter = TEST_DB_COUNTER.fetch_add(1, Ordering::SeqCst);
    let db_name = format!("file:test_db_{}?mode=memory&cache=shared", counter);

    let pool = SqlitePoolOptions::new()
        .max_connections(1) // SQLite in-memory works best with single connection
        .connect(&db_name)
        .await?;

    // Create tables compatible with both PostgreSQL and SQLite
    setup_test_schema(&pool).await?;

    Ok(pool)
}

#[cfg(any(test, debug_assertions))]
#[allow(dead_code)]
async fn setup_test_schema(pool: &Pool<Sqlite>) -> Result<()> {
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
            last_activity DATETIME
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

// Repository trait for database operations
#[allow(dead_code)]
pub trait Repository {
    type Entity;
    type CreateData;
    type UpdateData;

    fn create(&self, data: Self::CreateData) -> impl std::future::Future<Output = Result<Self::Entity>> + Send;
    fn find_by_id(&self, id: uuid::Uuid) -> impl std::future::Future<Output = Result<Option<Self::Entity>>> + Send;
    fn update(
        &self,
        id: uuid::Uuid,
        data: Self::UpdateData,
    ) -> impl std::future::Future<Output = Result<Self::Entity>> + Send;
    fn delete(&self, id: uuid::Uuid) -> impl std::future::Future<Output = Result<()>> + Send;
    fn list(
        &self,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> impl std::future::Future<Output = Result<Vec<Self::Entity>>> + Send;
}
