use super::branch_processor::CrawlProgress;
use crate::models::{Repository, RepositoryType};
use crate::repositories::RepositoryRepository;
use crate::services::encryption::EncryptionService;
use crate::services::gitlab::GitLabService;
use crate::services::progress::ProgressTracker;
use crate::services::search::SearchService;
use anyhow::{anyhow, Result};
use sqlx::{Pool, Postgres};
use std::path::PathBuf;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use uuid::Uuid;

/// GitLab-specific crawler operations
pub struct GitLabCrawler {
    database: Pool<Postgres>,
    search_service: Arc<SearchService>,
    progress_tracker: Arc<ProgressTracker>,
    encryption_service: Arc<EncryptionService>,
    temp_dir: PathBuf,
}

impl GitLabCrawler {
    pub fn new(
        database: Pool<Postgres>,
        search_service: Arc<SearchService>,
        progress_tracker: Arc<ProgressTracker>,
        encryption_service: Arc<EncryptionService>,
        temp_dir: PathBuf,
    ) -> Self {
        Self { database, search_service, progress_tracker, encryption_service, temp_dir }
    }

    /// Crawl a GitLab repository by discovering all sub-projects and cloning them
    pub async fn crawl_gitlab_repository(
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
        let gitlab_crawl_start_time = std::time::Instant::now();
        let repo_repo = RepositoryRepository::new(self.database.clone());

        info!("Starting GitLab discovery for repository: {}", repository.name);

        // Mark crawl as started in database
        repo_repo.start_crawl(repository.id, None).await?;

        // Delete all existing documents for this repository before crawling
        // This ensures no duplicates when re-crawling
        match self.search_service.delete_project_documents(&repository.name).await {
            Ok(deleted_count) => {
                if deleted_count > 0 {
                    info!(
                        "Deleted {} existing documents for GitLab repository {} before crawling",
                        deleted_count, repository.name
                    );
                }
            }
            Err(e) => {
                warn!(
                    "Failed to delete existing documents for GitLab repository {}: {}",
                    repository.name, e
                );
                // Continue anyway - the upsert should handle duplicates
            }
        }

        // Extract and decrypt access token from repository
        let encrypted_token =
            repository.access_token.as_ref().ok_or_else(|| anyhow!("GitLab repository missing access token"))?;

        let access_token = self
            .encryption_service
            .decrypt(encrypted_token)
            .map_err(|e| anyhow!("Failed to decrypt GitLab access token: {}", e))?;

        // Use URL if provided, otherwise default to gitlab.com
        let gitlab_url = if repository.url.is_empty() || repository.url == "placeholder" {
            "https://gitlab.com".to_string()
        } else {
            repository.url.clone()
        };

        self.progress_tracker.update_status(repository.id, crate::services::progress::CrawlStatus::Cloning).await;

        // Test GitLab token first
        let gitlab_service = GitLabService::new();
        info!("Testing GitLab token for repository: {}", repository.name);
        match gitlab_service.test_token(&gitlab_url, &access_token).await {
            Ok(true) => info!("GitLab token is valid"),
            Ok(false) => {
                let error_msg = "GitLab token is invalid or expired";
                error!(
                    "GitLab token validation failed for repository {}: {}",
                    repository.name, error_msg
                );
                self.progress_tracker.set_error(repository.id, error_msg.to_string()).await;
                // Mark crawl as failed in database
                let _ = repo_repo.fail_crawl(repository.id).await;
                cleanup_token_fn(repository.id).await;
                return Err(anyhow!(error_msg));
            }
            Err(e) => {
                let error_msg = format!("Failed to test GitLab token: {}", e);
                error!(
                    "GitLab token test error for repository {}: {}",
                    repository.name, error_msg
                );
                self.progress_tracker.set_error(repository.id, error_msg.clone()).await;
                // Mark crawl as failed in database
                let _ = repo_repo.fail_crawl(repository.id).await;
                cleanup_token_fn(repository.id).await;
                return Err(anyhow!(error_msg));
            }
        }

        // Discover GitLab projects
        info!(
            "Discovering GitLab projects for repository: {} with URL: {}",
            repository.name, gitlab_url
        );
        let projects = match gitlab_service
            .discover_projects(&gitlab_url, &access_token, repository.gitlab_namespace.as_deref())
            .await
        {
            Ok(projects) => projects,
            Err(e) => {
                let error_msg = format!("Failed to discover GitLab projects: {}", e);
                error!(
                    "GitLab discovery error for repository {}: {}",
                    repository.name, error_msg
                );
                self.progress_tracker.set_error(repository.id, error_msg.clone()).await;
                // Mark crawl as failed in database
                let _ = repo_repo.fail_crawl(repository.id).await;
                cleanup_token_fn(repository.id).await;
                return Err(anyhow!(error_msg));
            }
        };

        // Filter out excluded projects using repository-specific exclusions
        let excluded_projects: Vec<String> = repository
            .gitlab_excluded_projects
            .as_ref()
            .map(|s| s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
            .unwrap_or_default();
        let excluded_patterns: Vec<String> = repository
            .gitlab_excluded_patterns
            .as_ref()
            .map(|s| s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
            .unwrap_or_default();

        let filtered_projects =
            gitlab_service.filter_excluded_projects_with_config(projects, &excluded_projects, &excluded_patterns);

        if filtered_projects.is_empty() {
            let error_msg = "No accessible GitLab projects found after exclusion filtering";
            self.progress_tracker.set_error(repository.id, error_msg.to_string()).await;
            // Mark crawl as failed in database
            let _ = repo_repo.fail_crawl(repository.id).await;
            cleanup_token_fn(repository.id).await;
            return Err(anyhow!(error_msg));
        }

        info!(
            "Discovered {} GitLab projects for repository {} (after exclusion filtering)",
            filtered_projects.len(),
            repository.name
        );

        // Initialize hierarchical progress tracking for GitLab
        self.progress_tracker.set_gitlab_projects_total(repository.id, filtered_projects.len()).await;

        // Create base directory for this GitLab repository
        let base_repo_path = self.temp_dir.join(format!("{}-{}", repository.name, repository.id));
        std::fs::create_dir_all(&base_repo_path)?;

        let mut total_files_processed = 0;
        let mut total_files_indexed = 0;
        let mut all_errors = Vec::new();

        // Process each discovered project
        for (project_index, project) in filtered_projects.iter().enumerate() {
            // Update progress in database before processing each project
            repo_repo.update_crawl_progress(repository.id, Some(project.path_with_namespace.clone())).await?;
            // Check for cancellation before each project
            if cancellation_token.is_cancelled() {
                self.progress_tracker.cancel_crawl(repository.id).await;
                cleanup_token_fn(repository.id).await;
                return Ok(());
            }

            info!(
                "Processing GitLab project {}/{}: {}",
                project_index + 1,
                filtered_projects.len(),
                project.path_with_namespace
            );

            // Update current project in progress tracker
            self.progress_tracker
                .set_current_gitlab_project(repository.id, Some(project.path_with_namespace.clone()))
                .await;

            // Create sub-directory for this project
            let project_path = base_repo_path.join(&project.path_with_namespace);

            // Create a temporary repository object for this project
            let project_repository = Repository {
                id: repository.id,                         // Use same ID so it's grouped under the same repository
                name: project.path_with_namespace.clone(), // Use full project path as name
                url: project.http_url_to_repo.clone(),
                repository_type: RepositoryType::Git, // Treat as Git for cloning
                branch: project.default_branch.clone(),
                enabled: repository.enabled,
                access_token: repository.access_token.clone(),
                gitlab_namespace: repository.gitlab_namespace.clone(),
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
                gitlab_excluded_projects: repository.gitlab_excluded_projects.clone(),
                gitlab_excluded_patterns: repository.gitlab_excluded_patterns.clone(),
                github_namespace: repository.github_namespace.clone(),
                github_excluded_repositories: repository.github_excluded_repositories.clone(),
                github_excluded_patterns: repository.github_excluded_patterns.clone(),
                crawl_state: repository.crawl_state.clone(),
                last_processed_project: repository.last_processed_project.clone(),
                crawl_started_at: repository.crawl_started_at,
            };

            // Clone this specific project
            match clone_or_update_fn(&project_repository, &project_path).await {
                Ok(_) => {
                    // Create progress tracker for this project
                    let mut project_progress =
                        CrawlProgress { files_processed: 0, files_indexed: 0, errors: Vec::new() };

                    // Process files in this project with hierarchical tracking
                    match process_files_fn(
                        &project_repository,
                        &project_path,
                        &mut project_progress,
                        &cancellation_token,
                        repository.id,
                        &repository.name, // Pass parent repository name
                    )
                    .await
                    {
                        Ok(()) => {
                            info!("Successfully processed GitLab project: {}", project.path_with_namespace);
                            total_files_processed += project_progress.files_processed;
                            total_files_indexed += project_progress.files_indexed;
                            all_errors.extend(project_progress.errors);
                        }
                        Err(e) => {
                            let error_msg = format!(
                                "Failed to process files for project {}: {}",
                                project.path_with_namespace, e
                            );
                            warn!("{}", error_msg);
                            all_errors.push(error_msg);
                        }
                    }

                    // Complete this project in the tracker
                    self.progress_tracker.complete_current_gitlab_project(repository.id).await;
                }
                Err(e) => {
                    let error_msg = format!("Failed to clone GitLab project {}: {}", project.path_with_namespace, e);
                    warn!("{}", error_msg);
                    all_errors.push(error_msg);
                }
            }
        }

        // Update final progress
        self.progress_tracker.update_progress(repository.id, total_files_processed, None, total_files_indexed).await;

        if !all_errors.is_empty() {
            let combined_errors = all_errors.join("; ");
            self.progress_tracker.set_error(repository.id, format!("Some projects failed: {}", combined_errors)).await;
        }

        // Note: Tantivy commit is now done once at the end in crawler_service, not here, for better performance

        // Update repository crawl time with duration
        let gitlab_crawl_duration_seconds = gitlab_crawl_start_time.elapsed().as_secs() as i32;
        update_crawl_time_fn(repository.id, Some(gitlab_crawl_duration_seconds)).await?;

        // Mark crawl as completed in database
        repo_repo.complete_crawl(repository.id).await?;

        // Complete the crawl
        self.progress_tracker.complete_crawl(repository.id).await;
        cleanup_token_fn(repository.id).await;

        info!("Completed GitLab repository crawl for: {}", repository.name);
        Ok(())
    }

    /// Resume a GitLab repository crawl from a specific project
    pub async fn resume_gitlab_repository_crawl(
        &self,
        repository: &Repository,
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
        let repo_repo = RepositoryRepository::new(self.database.clone());
        let gitlab_crawl_start_time = std::time::Instant::now();

        info!(
            "Resuming GitLab crawl for repository: {} from project: {:?}",
            repository.name, repository.last_processed_project
        );

        // Extract and decrypt access token
        let encrypted_token =
            repository.access_token.as_ref().ok_or_else(|| anyhow!("GitLab repository missing access token"))?;

        let access_token = self
            .encryption_service
            .decrypt(encrypted_token)
            .map_err(|e| anyhow!("Failed to decrypt GitLab access token: {}", e))?;

        let gitlab_url = if repository.url.is_empty() || repository.url == "placeholder" {
            "https://gitlab.com".to_string()
        } else {
            repository.url.clone()
        };

        // Create cancellation token
        let cancellation_token = CancellationToken::new();

        // Start progress tracking
        self.progress_tracker.start_crawl(repository.id, repository.name.clone()).await;

        // Discover projects again
        let gitlab_service = GitLabService::new();
        let projects = gitlab_service
            .discover_projects(&gitlab_url, &access_token, repository.gitlab_namespace.as_deref())
            .await?;

        // Filter out excluded projects using repository-specific exclusions
        let excluded_projects: Vec<String> = repository
            .gitlab_excluded_projects
            .as_ref()
            .map(|s| s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
            .unwrap_or_default();
        let excluded_patterns: Vec<String> = repository
            .gitlab_excluded_patterns
            .as_ref()
            .map(|s| s.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
            .unwrap_or_default();

        let filtered_projects =
            gitlab_service.filter_excluded_projects_with_config(projects, &excluded_projects, &excluded_patterns);

        // Find where to resume from
        let resume_index = if let Some(ref last_project) = repository.last_processed_project {
            filtered_projects
                .iter()
                .position(|p| p.path_with_namespace == *last_project)
                .map(|i| i + 1) // Start from the next project
                .unwrap_or(0) // If not found, start from beginning
        } else {
            0 // Start from beginning if no last project
        };

        info!(
            "Resuming GitLab crawl from project index: {} / {}",
            resume_index,
            filtered_projects.len()
        );

        // Initialize progress tracking
        self.progress_tracker.set_gitlab_projects_total(repository.id, filtered_projects.len()).await;

        // Create base directory
        let base_repo_path = self.temp_dir.join(format!("{}-{}", repository.name, repository.id));
        std::fs::create_dir_all(&base_repo_path)?;

        let mut total_files_processed = 0;
        let mut total_files_indexed = 0;
        let mut all_errors = Vec::new();

        // Process projects starting from resume index
        for (project_index, project) in filtered_projects.iter().enumerate().skip(resume_index) {
            // Check for cancellation
            if cancellation_token.is_cancelled() {
                self.progress_tracker.cancel_crawl(repository.id).await;
                cleanup_token_fn(repository.id).await;
                return Ok(());
            }

            info!(
                "Resuming GitLab project {}/{}: {}",
                project_index + 1,
                filtered_projects.len(),
                project.path_with_namespace
            );

            // Update progress in database
            repo_repo.update_crawl_progress(repository.id, Some(project.path_with_namespace.clone())).await?;

            // Update current project in progress tracker
            self.progress_tracker
                .set_current_gitlab_project(repository.id, Some(project.path_with_namespace.clone()))
                .await;

            // Process the project (same logic as normal crawl)
            let project_path = base_repo_path.join(&project.path_with_namespace);

            let project_repository = Repository {
                id: repository.id,
                name: project.path_with_namespace.clone(),
                url: project.http_url_to_repo.clone(),
                repository_type: RepositoryType::Git,
                branch: project.default_branch.clone(),
                enabled: repository.enabled,
                access_token: repository.access_token.clone(),
                gitlab_namespace: repository.gitlab_namespace.clone(),
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
                gitlab_excluded_projects: repository.gitlab_excluded_projects.clone(),
                gitlab_excluded_patterns: repository.gitlab_excluded_patterns.clone(),
                github_namespace: repository.github_namespace.clone(),
                github_excluded_repositories: repository.github_excluded_repositories.clone(),
                github_excluded_patterns: repository.github_excluded_patterns.clone(),
                crawl_state: repository.crawl_state.clone(),
                last_processed_project: repository.last_processed_project.clone(),
                crawl_started_at: repository.crawl_started_at,
            };

            // Clone and process this project
            match clone_or_update_fn(&project_repository, &project_path).await {
                Ok(_) => {
                    let mut project_progress =
                        CrawlProgress { files_processed: 0, files_indexed: 0, errors: Vec::new() };

                    match process_files_fn(
                        &project_repository,
                        &project_path,
                        &mut project_progress,
                        &cancellation_token,
                        repository.id,
                        &repository.name, // Pass parent repository name
                    )
                    .await
                    {
                        Ok(()) => {
                            info!(
                                "Successfully resumed and processed GitLab project: {}",
                                project.path_with_namespace
                            );
                            total_files_processed += project_progress.files_processed;
                            total_files_indexed += project_progress.files_indexed;
                            all_errors.extend(project_progress.errors);
                        }
                        Err(e) => {
                            let error_msg = format!(
                                "Failed to process files for resumed project {}: {}",
                                project.path_with_namespace, e
                            );
                            warn!("{}", error_msg);
                            all_errors.push(error_msg);
                        }
                    }

                    self.progress_tracker.complete_current_gitlab_project(repository.id).await;
                }
                Err(e) => {
                    let error_msg = format!(
                        "Failed to clone resumed GitLab project {}: {}",
                        project.path_with_namespace, e
                    );
                    warn!("{}", error_msg);
                    all_errors.push(error_msg);
                }
            }
        }

        // Complete the resumed crawl
        self.progress_tracker.update_progress(repository.id, total_files_processed, None, total_files_indexed).await;

        if !all_errors.is_empty() {
            let combined_errors = all_errors.join("; ");
            self.progress_tracker
                .set_error(
                    repository.id,
                    format!("Some resumed projects failed: {}", combined_errors),
                )
                .await;
        }

        // Update repository crawl time and complete
        let gitlab_crawl_duration_seconds = gitlab_crawl_start_time.elapsed().as_secs() as i32;
        update_crawl_time_fn(repository.id, Some(gitlab_crawl_duration_seconds)).await?;

        // Mark crawl as completed in database
        repo_repo.complete_crawl(repository.id).await?;

        self.progress_tracker.complete_crawl(repository.id).await;
        cleanup_token_fn(repository.id).await;

        info!("Completed resumed GitLab repository crawl for: {}", repository.name);
        Ok(())
    }
}
