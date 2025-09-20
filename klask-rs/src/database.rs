use anyhow::Result;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

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

// For testing, we'll implement a mock database later
#[cfg(test)]
impl Database {
    pub async fn new_test() -> Result<Self> {
        // For now, we'll just use the regular PostgreSQL connection for tests
        // In a real implementation, we'd use an in-memory database or test container
        Self::new("postgres://test:test@localhost/test", 1).await
    }
}

// Repository trait for database operations
#[allow(dead_code)]
pub trait Repository {
    type Entity;
    type CreateData;
    type UpdateData;

    fn create(&self, data: Self::CreateData) -> impl std::future::Future<Output = Result<Self::Entity>> + Send;
    fn find_by_id(&self, id: uuid::Uuid) -> impl std::future::Future<Output = Result<Option<Self::Entity>>> + Send;
    fn update(&self, id: uuid::Uuid, data: Self::UpdateData) -> impl std::future::Future<Output = Result<Self::Entity>> + Send;
    fn delete(&self, id: uuid::Uuid) -> impl std::future::Future<Output = Result<()>> + Send;
    fn list(&self, limit: Option<u32>, offset: Option<u32>) -> impl std::future::Future<Output = Result<Vec<Self::Entity>>> + Send;
}
