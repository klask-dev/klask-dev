use crate::models::Repository;
use anyhow::Result;
use sqlx::PgPool;
use uuid::Uuid;

pub struct RepositoryRepository {
    pool: PgPool,
}

impl RepositoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_repository(&self, repository: &Repository) -> Result<Repository> {
        let result = sqlx::query_as::<_, Repository>(
            "INSERT INTO repositories (id, name, url, repository_type, branch, enabled, access_token, gitlab_namespace, is_group, auto_crawl_enabled, cron_schedule, next_crawl_at, crawl_frequency_hours, max_crawl_duration_minutes, gitlab_excluded_projects, gitlab_excluded_patterns, github_namespace, github_excluded_repositories, github_excluded_patterns, crawl_state, last_processed_project, crawl_started_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22) RETURNING id, name, url, repository_type, branch, enabled, access_token, gitlab_namespace, is_group, last_crawled, created_at, updated_at, auto_crawl_enabled, cron_schedule, next_crawl_at, crawl_frequency_hours, max_crawl_duration_minutes, last_crawl_duration_seconds, gitlab_excluded_projects, gitlab_excluded_patterns, github_namespace, github_excluded_repositories, github_excluded_patterns, crawl_state, last_processed_project, crawl_started_at"
        )
        .bind(repository.id)
        .bind(&repository.name)
        .bind(&repository.url)
        .bind(&repository.repository_type)
        .bind(&repository.branch)
        .bind(repository.enabled)
        .bind(&repository.access_token)
        .bind(&repository.gitlab_namespace)
        .bind(repository.is_group)
        .bind(repository.auto_crawl_enabled)
        .bind(&repository.cron_schedule)
        .bind(repository.next_crawl_at)
        .bind(repository.crawl_frequency_hours)
        .bind(repository.max_crawl_duration_minutes)
        .bind(&repository.gitlab_excluded_projects)
        .bind(&repository.gitlab_excluded_patterns)
        .bind(&repository.github_namespace)
        .bind(&repository.github_excluded_repositories)
        .bind(&repository.github_excluded_patterns)
        .bind(&repository.crawl_state)
        .bind(&repository.last_processed_project)
        .bind(repository.crawl_started_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn get_repository(&self, id: Uuid) -> Result<Option<Repository>> {
        let repository = sqlx::query_as::<_, Repository>(
            "SELECT id, name, url, repository_type, branch, enabled, access_token, gitlab_namespace, is_group, last_crawled, created_at, updated_at, auto_crawl_enabled, cron_schedule, next_crawl_at, crawl_frequency_hours, max_crawl_duration_minutes, last_crawl_duration_seconds, gitlab_excluded_projects, gitlab_excluded_patterns, github_namespace, github_excluded_repositories, github_excluded_patterns, crawl_state, last_processed_project, crawl_started_at FROM repositories WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(repository)
    }

    pub async fn list_repositories(&self) -> Result<Vec<Repository>> {
        let repositories = sqlx::query_as::<_, Repository>(
            "SELECT id, name, url, repository_type, branch, enabled, access_token, gitlab_namespace, is_group, last_crawled, created_at, updated_at, auto_crawl_enabled, cron_schedule, next_crawl_at, crawl_frequency_hours, max_crawl_duration_minutes, last_crawl_duration_seconds, gitlab_excluded_projects, gitlab_excluded_patterns, github_namespace, github_excluded_repositories, github_excluded_patterns, crawl_state, last_processed_project, crawl_started_at FROM repositories ORDER BY created_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(repositories)
    }

    #[allow(dead_code)]
    pub async fn update_last_crawled(&self, id: Uuid) -> Result<()> {
        sqlx::query("UPDATE repositories SET last_crawled = NOW(), updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_repository(&self, id: Uuid, repository: &Repository) -> Result<Repository> {
        let result = sqlx::query_as::<_, Repository>(
            "UPDATE repositories SET name = $2, url = $3, repository_type = $4, branch = $5, enabled = $6, access_token = $7, gitlab_namespace = $8, is_group = $9, auto_crawl_enabled = $10, cron_schedule = $11, next_crawl_at = $12, crawl_frequency_hours = $13, max_crawl_duration_minutes = $14, gitlab_excluded_projects = $15, gitlab_excluded_patterns = $16, github_namespace = $17, github_excluded_repositories = $18, github_excluded_patterns = $19, crawl_state = $20, last_processed_project = $21, crawl_started_at = $22, updated_at = NOW() WHERE id = $1 RETURNING id, name, url, repository_type, branch, enabled, access_token, gitlab_namespace, is_group, last_crawled, created_at, updated_at, auto_crawl_enabled, cron_schedule, next_crawl_at, crawl_frequency_hours, max_crawl_duration_minutes, last_crawl_duration_seconds, gitlab_excluded_projects, gitlab_excluded_patterns, github_namespace, github_excluded_repositories, github_excluded_patterns, crawl_state, last_processed_project, crawl_started_at"
        )
        .bind(id)
        .bind(&repository.name)
        .bind(&repository.url)
        .bind(&repository.repository_type)
        .bind(&repository.branch)
        .bind(repository.enabled)
        .bind(&repository.access_token)
        .bind(&repository.gitlab_namespace)
        .bind(repository.is_group)
        .bind(repository.auto_crawl_enabled)
        .bind(&repository.cron_schedule)
        .bind(repository.next_crawl_at)
        .bind(repository.crawl_frequency_hours)
        .bind(repository.max_crawl_duration_minutes)
        .bind(&repository.gitlab_excluded_projects)
        .bind(&repository.gitlab_excluded_patterns)
        .bind(&repository.github_namespace)
        .bind(&repository.github_excluded_repositories)
        .bind(&repository.github_excluded_patterns)
        .bind(&repository.crawl_state)
        .bind(&repository.last_processed_project)
        .bind(repository.crawl_started_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_repository(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM repositories WHERE id = $1").bind(id).execute(&self.pool).await?;

        Ok(())
    }

    pub async fn find_all(&self) -> Result<Vec<Repository>> {
        self.list_repositories().await
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Repository>> {
        self.get_repository(id).await
    }

    #[allow(dead_code)]
    pub async fn update_schedule(
        &self,
        id: Uuid,
        auto_crawl_enabled: bool,
        cron_schedule: Option<String>,
        crawl_frequency_hours: Option<i32>,
        max_crawl_duration_minutes: Option<i32>,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE repositories SET auto_crawl_enabled = $2, cron_schedule = $3, crawl_frequency_hours = $4, max_crawl_duration_minutes = $5, updated_at = NOW() WHERE id = $1"
        )
        .bind(id)
        .bind(auto_crawl_enabled)
        .bind(cron_schedule)
        .bind(crawl_frequency_hours)
        .bind(max_crawl_duration_minutes)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn find_scheduled_repositories(&self) -> Result<Vec<Repository>> {
        let repositories = sqlx::query_as::<_, Repository>(
            "SELECT id, name, url, repository_type, branch, enabled, access_token, gitlab_namespace, is_group, last_crawled, created_at, updated_at, auto_crawl_enabled, cron_schedule, next_crawl_at, crawl_frequency_hours, max_crawl_duration_minutes, last_crawl_duration_seconds, gitlab_excluded_projects, gitlab_excluded_patterns, github_namespace, github_excluded_repositories, github_excluded_patterns, crawl_state, last_processed_project, crawl_started_at FROM repositories WHERE auto_crawl_enabled = true ORDER BY next_crawl_at ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(repositories)
    }

    // Crawl resumption methods
    pub async fn start_crawl(&self, repository_id: Uuid, last_processed_project: Option<String>) -> Result<()> {
        sqlx::query(
            "UPDATE repositories SET crawl_state = 'in_progress', last_processed_project = $2, crawl_started_at = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind(repository_id)
        .bind(last_processed_project)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn complete_crawl(&self, repository_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE repositories SET crawl_state = 'idle', last_processed_project = NULL, crawl_started_at = NULL, last_crawled = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind(repository_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn fail_crawl(&self, repository_id: Uuid) -> Result<()> {
        sqlx::query("UPDATE repositories SET crawl_state = 'failed', updated_at = NOW() WHERE id = $1")
            .bind(repository_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn update_crawl_progress(
        &self,
        repository_id: Uuid,
        last_processed_project: Option<String>,
    ) -> Result<()> {
        sqlx::query("UPDATE repositories SET last_processed_project = $2, updated_at = NOW() WHERE id = $1")
            .bind(repository_id)
            .bind(last_processed_project)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn find_incomplete_crawls(&self) -> Result<Vec<Repository>> {
        let repositories = sqlx::query_as::<_, Repository>(
            "SELECT id, name, url, repository_type, branch, enabled, access_token, gitlab_namespace, is_group, last_crawled, created_at, updated_at, auto_crawl_enabled, cron_schedule, next_crawl_at, crawl_frequency_hours, max_crawl_duration_minutes, last_crawl_duration_seconds, gitlab_excluded_projects, gitlab_excluded_patterns, github_namespace, github_excluded_repositories, github_excluded_patterns, crawl_state, last_processed_project, crawl_started_at FROM repositories WHERE crawl_state = 'in_progress' AND enabled = true ORDER BY crawl_started_at ASC"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(repositories)
    }

    pub async fn find_abandoned_crawls(&self, timeout_minutes: i64) -> Result<Vec<Repository>> {
        let repositories = sqlx::query_as::<_, Repository>(
            "SELECT id, name, url, repository_type, branch, enabled, access_token, gitlab_namespace, is_group, last_crawled, created_at, updated_at, auto_crawl_enabled, cron_schedule, next_crawl_at, crawl_frequency_hours, max_crawl_duration_minutes, last_crawl_duration_seconds, gitlab_excluded_projects, gitlab_excluded_patterns, github_namespace, github_excluded_repositories, github_excluded_patterns, crawl_state, last_processed_project, crawl_started_at FROM repositories WHERE crawl_state = 'in_progress' AND crawl_started_at < NOW() - INTERVAL '1 minute' * $1 ORDER BY crawl_started_at ASC"
        )
        .bind(timeout_minutes)
        .fetch_all(&self.pool)
        .await?;

        Ok(repositories)
    }
}
