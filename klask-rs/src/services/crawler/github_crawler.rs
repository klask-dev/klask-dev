use super::branch_processor::CrawlProgress;
use crate::models::{Repository, RepositoryType};
use crate::repositories::RepositoryRepository;
use crate::services::encryption::EncryptionService;
use crate::services::github::GitHubService;
use crate::services::progress::ProgressTracker;
use crate::services::search::SearchService;
use anyhow::{anyhow, Result};
use sqlx::{Pool, Postgres};
use std::path::PathBuf;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use uuid::Uuid;

/// GitHub-specific crawler operations
pub struct GitHubCrawler {
    database: Pool<Postgres>,
    search_service: Arc<SearchService>,
    progress_tracker: Arc<ProgressTracker>,
    encryption_service: Arc<EncryptionService>,
    temp_dir: PathBuf,
}

impl GitHubCrawler {
    pub fn new(
        database: Pool<Postgres>,
        search_service: Arc<SearchService>,
        progress_tracker: Arc<ProgressTracker>,
        encryption_service: Arc<EncryptionService>,
        temp_dir: PathBuf,
    ) -> Self {
        Self { database, search_service, progress_tracker, encryption_service, temp_dir }
    }

    /// Crawl a GitHub repository by discovering all sub-repositories and cloning them
    pub async fn crawl_github_repository(
        &self,
        repository: &Repository,
        cancellation_token: CancellationToken,
        clone_or_update_fn: impl Fn(
                &Repository,
                &std::path::Path,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<gix::Repository>> + Send>>
            + Send
            + Sync,
        process_files_fn: impl Fn(
                &Repository,
                &std::path::Path,
                &mut CrawlProgress,
                &CancellationToken,
                Uuid,
                &str,
            ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>
            + Send
            + Sync,
        update_crawl_time_fn: impl Fn(Uuid, Option<i32>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>
            + Send
            + Sync,
        cleanup_token_fn: impl Fn(Uuid) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync,
    ) -> Result<()> {
        let github_crawl_start_time = std::time::Instant::now();
        let repo_repo = RepositoryRepository::new(self.database.clone());

        info!("Starting GitHub discovery for repository: {}", repository.name);

        // Mark crawl as started in database
        repo_repo.start_crawl(repository.id, None).await?;

        // Delete all existing documents for this repository before crawling
        // This ensures no duplicates when re-crawling
        match self.search_service.delete_project_documents(&repository.name).await {
            Ok(deleted_count) => {
                if deleted_count > 0 {
                    info!(
                        "Deleted {} existing documents for GitHub repository {} before crawling",
                        deleted_count, repository.name
                    );
                }
            }
            Err(e) => {
                warn!(
                    "Failed to delete existing documents for GitHub repository {}: {}",
                    repository.name, e
                );
                // Continue anyway - the upsert should handle duplicates
            }
        }

        // Extract and decrypt access token from repository
        let encrypted_token =
            repository.access_token.as_ref().ok_or_else(|| anyhow!("GitHub repository missing access token"))?;

        let access_token = self
            .encryption_service
            .decrypt(encrypted_token)
            .map_err(|e| anyhow!("Failed to decrypt GitHub access token: {}", e))?;

        self.progress_tracker.update_status(repository.id, crate::services::progress::CrawlStatus::Cloning).await;

        // Test GitHub token first
        let github_service = GitHubService::new();
        info!("Testing GitHub token for repository: {}", repository.name);
        match github_service.test_token(&access_token).await {
            Ok(true) => info!("GitHub token is valid"),
            Ok(false) => {
                let error_msg = "GitHub token is invalid or expired";
                error!(
                    "GitHub token validation failed for repository {}: {}",
                    repository.name, error_msg
                );
                self.progress_tracker.set_error(repository.id, error_msg.to_string()).await;
                // Mark crawl as failed in database
                let _ = repo_repo.fail_crawl(repository.id).await;
                cleanup_token_fn(repository.id).await;
                return Err(anyhow!(error_msg));
            }
            Err(e) => {
                let error_msg = format!("Failed to test GitHub token: {}", e);
                error!(
                    "GitHub token test error for repository {}: {}",
                    repository.name, error_msg
                );
                self.progress_tracker.set_error(repository.id, error_msg.clone()).await;
                // Mark crawl as failed in database
                let _ = repo_repo.fail_crawl(repository.id).await;
                cleanup_token_fn(repository.id).await;
                return Err(anyhow!(error_msg));
            }
        }

        // Discover GitHub repositories
        info!("Discovering GitHub repositories for repository: {}", repository.name);
        let repositories =
            match github_service.discover_repositories(&access_token, repository.github_namespace.as_deref()).await {
                Ok(repos) => repos,
                Err(e) => {
                    let error_msg = format!("Failed to discover GitHub repositories: {}", e);
                    error!(
                        "GitHub discovery error for repository {}: {}",
                        repository.name, error_msg
                    );
                    self.progress_tracker.set_error(repository.id, error_msg.clone()).await;
                    // Mark crawl as failed in database
                    let _ = repo_repo.fail_crawl(repository.id).await;
                    cleanup_token_fn(repository.id).await;
                    return Err(anyhow!(error_msg));
                }
            };

        // Filter out excluded repositories using repository-specific exclusions
        let excluded_repositories: Vec<String> = repository
            .github_excluded_repositories
            .as_ref()
            .map(|s| s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
            .unwrap_or_default();
        let excluded_patterns: Vec<String> = repository
            .github_excluded_patterns
            .as_ref()
            .map(|s| s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
            .unwrap_or_default();

        let filtered_repositories = github_service.filter_excluded_repositories_with_config(
            repositories,
            &excluded_repositories,
            &excluded_patterns,
        );

        if filtered_repositories.is_empty() {
            let error_msg = "No accessible GitHub repositories found after exclusion filtering";
            self.progress_tracker.set_error(repository.id, error_msg.to_string()).await;
            // Mark crawl as failed in database
            let _ = repo_repo.fail_crawl(repository.id).await;
            cleanup_token_fn(repository.id).await;
            return Err(anyhow!(error_msg));
        }

        info!(
            "Discovered {} GitHub repositories for repository {} (after exclusion filtering)",
            filtered_repositories.len(),
            repository.name
        );

        // Initialize hierarchical progress tracking for GitHub
        self.progress_tracker.set_gitlab_projects_total(repository.id, filtered_repositories.len()).await;

        // Create base directory for this GitHub repository
        let base_repo_path = self.temp_dir.join(format!("{}-{}", repository.name, repository.id));
        std::fs::create_dir_all(&base_repo_path)?;

        let mut total_files_processed = 0;
        let mut total_files_indexed = 0;
        let mut all_errors = Vec::new();

        // Process each discovered repository
        for (repo_index, github_repo) in filtered_repositories.iter().enumerate() {
            // Update progress in database before processing each repository
            repo_repo.update_crawl_progress(repository.id, Some(github_repo.full_name.clone())).await?;

            // Check for cancellation before each repository
            if cancellation_token.is_cancelled() {
                self.progress_tracker.cancel_crawl(repository.id).await;
                cleanup_token_fn(repository.id).await;
                return Ok(());
            }

            info!(
                "Processing GitHub repository {}/{}: {}",
                repo_index + 1,
                filtered_repositories.len(),
                github_repo.full_name
            );

            // Update current project in progress tracker
            self.progress_tracker.set_current_gitlab_project(repository.id, Some(github_repo.full_name.clone())).await;

            // Create sub-directory for this repository
            let repo_path = base_repo_path.join(&github_repo.full_name);

            // Create a temporary repository object for this GitHub repo
            let temp_repository = Repository {
                id: repository.id,                   // Use same ID so it's grouped under the same repository
                name: github_repo.full_name.clone(), // Use full repo name
                url: github_repo.clone_url.clone(),
                repository_type: RepositoryType::Git, // Treat as Git for cloning
                branch: Some(github_repo.default_branch.clone()),
                enabled: repository.enabled,
                access_token: repository.access_token.clone(),
                gitlab_namespace: None,
                is_group: false,
                created_at: repository.created_at,
                updated_at: repository.updated_at,
                last_crawled: repository.last_crawled,
                auto_crawl_enabled: repository.auto_crawl_enabled,
                cron_schedule: repository.cron_schedule.clone(),
                next_crawl_at: repository.next_crawl_at,
                crawl_frequency_hours: repository.crawl_frequency_hours,
                max_crawl_duration_minutes: repository.max_crawl_duration_minutes,
                last_crawl_duration_seconds: repository.last_crawl_duration_seconds,
                gitlab_excluded_projects: None,
                gitlab_excluded_patterns: None,
                github_namespace: repository.github_namespace.clone(),
                github_excluded_repositories: repository.github_excluded_repositories.clone(),
                github_excluded_patterns: repository.github_excluded_patterns.clone(),
                crawl_state: repository.crawl_state.clone(),
                last_processed_project: repository.last_processed_project.clone(),
                crawl_started_at: repository.crawl_started_at,
            };

            // Clone this specific repository
            match clone_or_update_fn(&temp_repository, &repo_path).await {
                Ok(_) => {
                    // Create progress tracker for this repository
                    let mut repo_progress = CrawlProgress { files_processed: 0, files_indexed: 0, errors: Vec::new() };

                    // Process files in this repository with hierarchical tracking
                    match process_files_fn(
                        &temp_repository,
                        &repo_path,
                        &mut repo_progress,
                        &cancellation_token,
                        repository.id,
                        &repository.name, // Pass parent repository name
                    )
                    .await
                    {
                        Ok(()) => {
                            info!("Successfully processed GitHub repository: {}", github_repo.full_name);
                            total_files_processed += repo_progress.files_processed;
                            total_files_indexed += repo_progress.files_indexed;
                            all_errors.extend(repo_progress.errors);
                        }
                        Err(e) => {
                            let error_msg = format!(
                                "Failed to process files for repository {}: {}",
                                github_repo.full_name, e
                            );
                            warn!("{}", error_msg);
                            all_errors.push(error_msg);
                        }
                    }

                    // Complete this repository in the tracker
                    self.progress_tracker.complete_current_gitlab_project(repository.id).await;
                }
                Err(e) => {
                    let error_msg = format!("Failed to clone GitHub repository {}: {}", github_repo.full_name, e);
                    warn!("{}", error_msg);
                    all_errors.push(error_msg);
                }
            }
        }

        // Update final progress
        self.progress_tracker.update_progress(repository.id, total_files_processed, None, total_files_indexed).await;

        if !all_errors.is_empty() {
            let combined_errors = all_errors.join("; ");
            self.progress_tracker
                .set_error(repository.id, format!("Some repositories failed: {}", combined_errors))
                .await;
        }

        // Note: Tantivy commit is now done once at the end in crawler_service, not here, for better performance

        // Update repository crawl time with duration
        let github_crawl_duration_seconds = github_crawl_start_time.elapsed().as_secs() as i32;
        update_crawl_time_fn(repository.id, Some(github_crawl_duration_seconds)).await?;

        // Mark crawl as completed in database
        repo_repo.complete_crawl(repository.id).await?;

        // Complete the crawl
        self.progress_tracker.complete_crawl(repository.id).await;
        cleanup_token_fn(repository.id).await;

        info!("Completed GitHub repository crawl for: {}", repository.name);
        Ok(())
    }
}
