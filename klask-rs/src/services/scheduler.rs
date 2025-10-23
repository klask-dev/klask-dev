use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use croner::Cron;
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    models::repository::Repository, repositories::repository_repository::RepositoryRepository,
    services::crawler::CrawlerService,
};

/// Scheduled job handle for a repository
struct ScheduledJob {
    cron_expression: String,
    task_handle: JoinHandle<()>,
}

pub struct SchedulerService {
    pool: PgPool,
    crawler_service: Arc<CrawlerService>,
    // Map of repository_id -> ScheduledJob
    jobs: Arc<RwLock<HashMap<Uuid, ScheduledJob>>>,
}

impl SchedulerService {
    pub async fn new(pool: PgPool, crawler_service: Arc<CrawlerService>) -> Result<Self> {
        Ok(Self { pool, crawler_service, jobs: Arc::new(RwLock::new(HashMap::new())) })
    }

    /// Start the scheduler and load all scheduled repositories
    pub async fn start(&self) -> Result<()> {
        info!("Starting repository scheduler service");

        // Load all repositories with scheduling enabled
        self.reload_scheduled_repositories().await?;

        // TEMPORARILY DISABLED: Schedule a periodic task to reload repository schedules (every 5 minutes)
        // This was causing performance issues by spawning too many tasks
        // TODO: Fix the reload logic to properly clean up old tasks before creating new ones
        // self.schedule_periodic_reload().await?;

        info!("Repository scheduler service started successfully (periodic reload disabled)");
        Ok(())
    }

    /// Stop the scheduler service
    #[allow(dead_code)]
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping repository scheduler service");

        let mut jobs = self.jobs.write().await;
        for (repo_id, job) in jobs.drain() {
            debug!("Cancelling scheduled job for repository {}", repo_id);
            job.task_handle.abort();
        }

        info!("Repository scheduler service stopped");
        Ok(())
    }

    /// Schedule a repository for automatic crawling
    pub async fn schedule_repository(&self, repository: &Repository) -> Result<()> {
        if !repository.auto_crawl_enabled {
            debug!("Auto-crawl not enabled for repository {}", repository.id);
            return Ok(());
        }

        // Remove existing schedule if any
        self.unschedule_repository(repository.id).await?;

        // Get cron expression (either from cron_schedule or convert from frequency)
        let cron_expr = if let Some(ref cron_schedule) = repository.cron_schedule {
            cron_schedule.clone()
        } else if let Some(frequency_hours) = repository.crawl_frequency_hours {
            // Convert hours to cron expression: "0 0 */X * * *" (every X hours)
            format!("0 0 */{} * * *", frequency_hours)
        } else {
            warn!(
                "Repository {} has auto-crawl enabled but no schedule defined",
                repository.id
            );
            return Ok(());
        };

        // Parse and validate the cron expression (with 6-field format: seconds minutes hours day month weekday)
        // croner 3.0 uses parse() directly - validate by attempting to parse
        let _cron: croner::Cron =
            cron_expr.parse().with_context(|| format!("Failed to parse cron expression: {}", cron_expr))?;

        info!(
            "Scheduling repository {} ({}) with cron: {}",
            repository.name, repository.id, cron_expr
        );

        // Spawn the scheduling task
        let repo_id = repository.id;
        let repo_name = repository.name.clone();
        let crawler = self.crawler_service.clone();
        let pool = self.pool.clone();
        let cron_expr_clone = cron_expr.clone();

        let task_handle = tokio::spawn(async move {
            loop {
                // Calculate next run time
                // IMPORTANT: Always get the NEXT occurrence after now, not including current time
                let now = Utc::now();

                // Parse cron expression (we need to do this in the loop since cron is not Clone)
                let cron_result: Result<Cron, _> = cron_expr_clone.parse();

                let cron = match cron_result {
                    Ok(c) => c,
                    Err(e) => {
                        error!(
                            "Failed to parse cron expression '{}' for repository {}: {}",
                            cron_expr_clone, repo_id, e
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                        continue;
                    }
                };

                match cron.find_next_occurrence(&now, false) {
                    Ok(next_run) => {
                        // Update next_crawl_at in database
                        if let Err(e) = Self::update_next_crawl_time_static(&pool, repo_id, Some(next_run)).await {
                            error!("Failed to update next_crawl_at for repository {}: {}", repo_id, e);
                        }

                        // Calculate duration to sleep
                        let duration = (next_run - now).to_std().unwrap_or(std::time::Duration::from_secs(0));

                        debug!(
                            "Repository {} ({}) scheduled to run at {} (in {:?})",
                            repo_name, repo_id, next_run, duration
                        );

                        // Sleep until next run
                        tokio::time::sleep(duration).await;

                        // Execute the crawl
                        info!("Executing scheduled crawl for repository {} ({})", repo_name, repo_id);

                        // Load repository from database to check if auto_crawl is still enabled
                        let repo_repository = RepositoryRepository::new(pool.clone());
                        match repo_repository.find_by_id(repo_id).await {
                            Ok(Some(repository)) => {
                                // Check if auto_crawl is still enabled
                                if !repository.auto_crawl_enabled {
                                    info!(
                                        "Auto-crawl disabled for repository {} ({}), stopping scheduler task",
                                        repo_name, repo_id
                                    );
                                    break; // Exit the loop to stop this scheduler task
                                }

                                // Execute the crawl
                                match crawler.crawl_repository(&repository).await {
                                    Ok(_) => {
                                        info!("Scheduled crawl completed successfully for repository {}", repo_id);
                                    }
                                    Err(e) => {
                                        error!("Scheduled crawl failed for repository {}: {}", repo_id, e);
                                    }
                                }
                            }
                            Ok(None) => {
                                error!("Repository {} not found in database, stopping scheduler task", repo_id);
                                break; // Exit if repository was deleted
                            }
                            Err(e) => {
                                error!("Failed to load repository {} from database: {}", repo_id, e);
                                // Don't break here, just skip this iteration and try again next time
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to calculate next occurrence for repository {} with cron '{}': {}",
                            repo_id, cron_expr_clone, e
                        );
                        // Wait a bit before retrying
                        tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
                    }
                }
            }
        });

        // Store the job handle
        let job = ScheduledJob { cron_expression: cron_expr.clone(), task_handle };

        self.jobs.write().await.insert(repo_id, job);

        // Update next_crawl_at immediately
        self.update_next_crawl_time(repository.id, &cron_expr).await?;

        info!("Successfully scheduled repository {} with cron: {}", repo_id, cron_expr);

        Ok(())
    }

    /// Unschedule a repository from automatic crawling
    pub async fn unschedule_repository(&self, repository_id: Uuid) -> Result<()> {
        let mut jobs = self.jobs.write().await;

        if let Some(job) = jobs.remove(&repository_id) {
            debug!("Unscheduling repository {}", repository_id);
            job.task_handle.abort();

            // Clear next_crawl_at in database
            Self::update_next_crawl_time_static(&self.pool, repository_id, None).await?;
        }

        Ok(())
    }

    /// Get the next scheduled run time for a repository
    #[allow(dead_code)]
    pub async fn get_next_run_time(&self, repository_id: Uuid) -> Option<DateTime<Utc>> {
        let jobs = self.jobs.read().await;

        if let Some(job) = jobs.get(&repository_id) {
            // Parse the cron and get next occurrence
            if let Ok(cron) = job.cron_expression.parse::<Cron>() {
                if let Ok(next) = cron.find_next_occurrence(&Utc::now(), false) {
                    return Some(next);
                }
            }
        }

        None
    }

    /// Update the next crawl time for a repository in the database
    async fn update_next_crawl_time(&self, repository_id: Uuid, cron_expr: &str) -> Result<()> {
        let cron: Cron =
            cron_expr.parse().with_context(|| format!("Failed to parse cron expression: {}", cron_expr))?;

        let next_run = cron.find_next_occurrence(&Utc::now(), false).context("Failed to calculate next occurrence")?;

        Self::update_next_crawl_time_static(&self.pool, repository_id, Some(next_run)).await
    }

    /// Static helper to update next_crawl_at in database
    async fn update_next_crawl_time_static(
        pool: &PgPool,
        repository_id: Uuid,
        next_crawl_at: Option<DateTime<Utc>>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE repositories
            SET next_crawl_at = $1,
                updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(next_crawl_at)
        .bind(repository_id)
        .execute(pool)
        .await
        .context("Failed to update next_crawl_at")?;

        Ok(())
    }

    /// Reload all scheduled repositories from the database
    pub async fn reload_scheduled_repositories(&self) -> Result<()> {
        info!("Reloading scheduled repositories");

        let repo_repository = RepositoryRepository::new(self.pool.clone());
        let repositories = repo_repository.find_all().await.context("Failed to load repositories")?;

        let scheduled_count = repositories.iter().filter(|r| r.auto_crawl_enabled).count();

        info!(
            "Found {} repositories with auto-crawl enabled out of {}",
            scheduled_count,
            repositories.len()
        );

        for repository in repositories.iter().filter(|r| r.auto_crawl_enabled) {
            if let Err(e) = self.schedule_repository(repository).await {
                error!(
                    "Failed to schedule repository {} ({}): {}",
                    repository.name, repository.id, e
                );
            }
        }

        info!("Finished reloading scheduled repositories");
        Ok(())
    }

    /// Schedule a periodic task to reload repository schedules
    #[allow(dead_code)]
    async fn schedule_periodic_reload(&self) -> Result<()> {
        let self_clone = Arc::new(self.clone_for_reload());

        tokio::spawn(async move {
            let reload_interval = tokio::time::Duration::from_secs(5 * 60); // 5 minutes

            loop {
                tokio::time::sleep(reload_interval).await;

                info!("Performing periodic reload of scheduled repositories");

                if let Err(e) = self_clone.reload_scheduled_repositories().await {
                    error!("Periodic reload failed: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Helper to clone necessary fields for periodic reload
    #[allow(dead_code)]
    fn clone_for_reload(&self) -> Self {
        Self { pool: self.pool.clone(), crawler_service: self.crawler_service.clone(), jobs: self.jobs.clone() }
    }

    /// Get the current scheduler status
    pub async fn get_status(&self) -> SchedulerStatus {
        let jobs = self.jobs.read().await;

        // Get all repository info from database
        let repo_repository = RepositoryRepository::new(self.pool.clone());
        let all_repos = repo_repository.find_all().await.unwrap_or_default();

        // Count repos with auto_crawl enabled
        let auto_crawl_enabled_count = all_repos.iter().filter(|r| r.auto_crawl_enabled).count();

        let mut next_runs = Vec::new();
        for (repo_id, job) in jobs.iter() {
            // Find repository name from database
            let repo_name = all_repos
                .iter()
                .find(|r| r.id == *repo_id)
                .map(|r| r.name.clone())
                .unwrap_or_else(|| format!("Unknown ({})", repo_id));

            if let Ok(cron) = job.cron_expression.parse::<Cron>() {
                if let Ok(next_run) = cron.find_next_occurrence(&Utc::now(), false) {
                    next_runs.push(NextRun {
                        repository_id: *repo_id,
                        repository_name: repo_name,
                        next_run_at: next_run,
                        cron_expression: job.cron_expression.clone(),
                        schedule_expression: job.cron_expression.clone(),
                    });
                }
            }
        }

        // Sort by next run time
        next_runs.sort_by(|a, b| a.next_run_at.cmp(&b.next_run_at));

        let scheduled_count = jobs.len();

        SchedulerStatus {
            is_running: true,
            scheduled_jobs_count: scheduled_count,
            scheduled_repositories_count: scheduled_count,
            auto_crawl_enabled_count,
            next_runs,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct SchedulerStatus {
    pub is_running: bool,
    pub scheduled_jobs_count: usize,
    pub scheduled_repositories_count: usize, // Same as scheduled_jobs_count for compatibility
    pub auto_crawl_enabled_count: usize,
    pub next_runs: Vec<NextRun>,
}

#[derive(Debug, serde::Serialize)]
pub struct NextRun {
    pub repository_id: Uuid,
    pub repository_name: String,
    pub next_run_at: DateTime<Utc>,
    pub cron_expression: String,
    pub schedule_expression: String, // Same as cron_expression for compatibility
}
