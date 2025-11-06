use super::branch_processor::{BranchProcessor, CrawlProgress};
use super::file_processing::SUPPORTED_EXTENSIONS;
use super::git_operations::GitOperations;
use super::github_crawler::GitHubCrawler;
use super::gitlab_crawler::GitLabCrawler;
use crate::models::{Repository, RepositoryType};
use crate::repositories::RepositoryRepository;
use crate::services::encryption::EncryptionService;
use crate::services::progress::ProgressTracker;
use crate::services::search::SearchService;
use anyhow::{Result, anyhow};
use sha2::{Digest, Sha256};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Main crawler service that orchestrates all crawl operations
pub struct CrawlerService {
    database: Pool<Postgres>,
    search_service: Arc<SearchService>,
    progress_tracker: Arc<ProgressTracker>,
    encryption_service: Arc<EncryptionService>,
    pub temp_dir: PathBuf,
    cancellation_tokens: Arc<RwLock<HashMap<Uuid, CancellationToken>>>,
    // Specialized crawlers
    git_operations: GitOperations,
    branch_processor: BranchProcessor,
    gitlab_crawler: GitLabCrawler,
    github_crawler: GitHubCrawler,
}

impl CrawlerService {
    pub fn new(
        database: Pool<Postgres>,
        search_service: Arc<SearchService>,
        progress_tracker: Arc<ProgressTracker>,
        encryption_service: Arc<EncryptionService>,
        temp_dir: String,
    ) -> Result<Self> {
        let temp_dir = std::path::PathBuf::from(temp_dir);
        std::fs::create_dir_all(&temp_dir).map_err(|e| anyhow!("Failed to create temp directory: {}", e))?;

        // Create specialized crawlers
        let git_operations = GitOperations::new(encryption_service.clone());
        let branch_processor = BranchProcessor::new(search_service.clone(), progress_tracker.clone());
        let gitlab_crawler = GitLabCrawler::new(
            database.clone(),
            search_service.clone(),
            progress_tracker.clone(),
            encryption_service.clone(),
            temp_dir.clone(),
        );
        let github_crawler = GitHubCrawler::new(
            database.clone(),
            search_service.clone(),
            progress_tracker.clone(),
            encryption_service.clone(),
            temp_dir.clone(),
        );

        Ok(Self {
            database,
            search_service,
            progress_tracker,
            encryption_service,
            temp_dir,
            cancellation_tokens: Arc::new(RwLock::new(HashMap::new())),
            git_operations,
            branch_processor,
            gitlab_crawler,
            github_crawler,
        })
    }

    /// Generate a deterministic UUID for a file based on repository, specific branch, and path
    #[allow(dead_code)]
    fn generate_deterministic_file_id_with_branch(
        &self,
        repository: &Repository,
        relative_path: &str,
        branch_name: &str,
    ) -> Uuid {
        let mut hasher = Sha256::new();

        // Create deterministic input based on repository type
        let input = match repository.repository_type {
            RepositoryType::FileSystem => {
                // For FileSystem: hash of {repository.url}:{relative_path}
                format!("{}:{}", repository.url, relative_path)
            }
            RepositoryType::Git | RepositoryType::GitLab | RepositoryType::GitHub => {
                // For Git/GitLab/GitHub: hash of {repository.url}:{branch}:{relative_path}
                format!("{}:{}:{}", repository.url, branch_name, relative_path)
            }
        };

        hasher.update(input.as_bytes());
        let hash_bytes = hasher.finalize();

        // Convert SHA-256 hash to UUID format
        // Take first 16 bytes of the hash to create a UUID
        let mut uuid_bytes = [0u8; 16];
        uuid_bytes.copy_from_slice(&hash_bytes[..16]);

        // Set version to 4 (random) and variant bits according to RFC 4122
        uuid_bytes[6] = (uuid_bytes[6] & 0x0f) | 0x40; // Version 4
        uuid_bytes[8] = (uuid_bytes[8] & 0x3f) | 0x80; // Variant bits

        Uuid::from_bytes(uuid_bytes)
    }

    /// Main entry point for crawling a repository
    pub async fn crawl_repository(&self, repository: &Repository) -> Result<()> {
        let crawl_start_time = std::time::Instant::now();
        let repo_repo = RepositoryRepository::new(self.database.clone());

        info!(
            "Starting crawl for repository: {} ({}) - Type: {:?}",
            repository.name, repository.url, repository.repository_type
        );

        // Mark crawl as started in database
        repo_repo.start_crawl(repository.id, None).await?;

        // Delete all existing documents for this repository/project before crawling
        // This ensures no duplicates when re-crawling
        match self.search_service.delete_project_documents(&repository.name).await {
            Ok(deleted_count) => {
                if deleted_count > 0 {
                    info!(
                        "Deleted {} existing documents for repository {} before crawling",
                        deleted_count, repository.name
                    );
                }
            }
            Err(e) => {
                warn!(
                    "Failed to delete existing documents for repository {}: {}",
                    repository.name, e
                );
                // Continue anyway - the upsert should handle duplicates
            }
        }

        // Create cancellation token for this crawl
        let cancellation_token = CancellationToken::new();
        {
            let mut tokens = self.cancellation_tokens.write().await;
            tokens.insert(repository.id, cancellation_token.clone());
        }

        // Start tracking progress
        self.progress_tracker.start_crawl(repository.id, repository.name.clone()).await;
        self.progress_tracker.update_status(repository.id, crate::services::progress::CrawlStatus::Starting).await;

        // Check for cancellation before starting main work
        if cancellation_token.is_cancelled() {
            self.progress_tracker.cancel_crawl(repository.id).await;
            self.cleanup_cancellation_token(repository.id).await;
            return Ok(());
        }

        // Initialize progress for all repository types
        let mut progress = CrawlProgress { files_processed: 0, files_indexed: 0, errors: Vec::new() };

        let repo_path_git = match repository.repository_type {
            RepositoryType::Git => {
                let temp_path = self.temp_dir.join(format!("{}-{}", repository.name, repository.id));

                // For Git repositories, use temp directory with cloning
                self.progress_tracker
                    .update_status(repository.id, crate::services::progress::CrawlStatus::Cloning)
                    .await;

                // Try to clone the repository, handle errors gracefully
                match self.git_operations.clone_or_update_repository(repository, &temp_path).await {
                    Ok(_git_repo) => temp_path,
                    Err(e) => {
                        let error_msg = format!("Failed to clone/update repository: {}", e);
                        error!("Crawl error for repository {}: {}", repository.name, error_msg);
                        self.progress_tracker.set_error(repository.id, error_msg.clone()).await;
                        // Mark crawl as failed in database
                        let _ = repo_repo.fail_crawl(repository.id).await;
                        self.cleanup_cancellation_token(repository.id).await;
                        return Err(anyhow!(error_msg));
                    }
                }
            }
            RepositoryType::FileSystem => {
                // For filesystem repositories, use the URL as the direct path
                PathBuf::from(&repository.url)
            }
            _ => PathBuf::new(),
        };

        match repository.repository_type {
            RepositoryType::FileSystem | RepositoryType::Git => {
                // Validate that the path exists
                if !repo_path_git.exists() {
                    let error_msg = format!("Repository path does not exist: {:?}", repo_path_git);
                    self.progress_tracker.set_error(repository.id, error_msg.clone()).await;
                    // Mark crawl as failed in database
                    let _ = repo_repo.fail_crawl(repository.id).await;
                    self.cleanup_cancellation_token(repository.id).await;
                    return Err(anyhow!(error_msg));
                }

                if !repo_path_git.is_dir() {
                    let error_msg = format!("Repository path is not a directory: {:?}", repo_path_git);
                    self.progress_tracker.set_error(repository.id, error_msg.clone()).await;
                    // Mark crawl as failed in database
                    let _ = repo_repo.fail_crawl(repository.id).await;
                    self.cleanup_cancellation_token(repository.id).await;
                    return Err(anyhow!(error_msg));
                }

                // Update status to processing
                self.progress_tracker
                    .update_status(repository.id, crate::services::progress::CrawlStatus::Processing)
                    .await;

                // Check for cancellation before processing
                if cancellation_token.is_cancelled() {
                    self.progress_tracker.cancel_crawl(repository.id).await;
                    self.cleanup_cancellation_token(repository.id).await;
                    return Ok(());
                }

                self.process_repository_files(repository, &repo_path_git, &mut progress, &cancellation_token).await?;

                // Check for cancellation before indexing
                if cancellation_token.is_cancelled() {
                    self.progress_tracker.cancel_crawl(repository.id).await;
                    self.cleanup_cancellation_token(repository.id).await;
                    return Ok(());
                }

                // Update status to indexing
                self.progress_tracker
                    .update_status(repository.id, crate::services::progress::CrawlStatus::Indexing)
                    .await;
            }
            RepositoryType::GitLab => {
                // For GitLab repositories, discover and clone all sub-projects
                // Create closures for GitLab crawler callbacks
                let clone_or_update_fn = |repo: &Repository, path: &Path| {
                    let repo = repo.clone();
                    let path = path.to_owned();
                    let git_ops = self.git_operations.clone();
                    Box::pin(async move { git_ops.clone_or_update_repository(&repo, &path).await })
                        as std::pin::Pin<Box<dyn std::future::Future<Output = Result<gix::Repository>> + Send>>
                };

                let process_files_fn = |repo: &Repository,
                                        path: &Path,
                                        progress: &mut CrawlProgress,
                                        token: &CancellationToken,
                                        parent_id: Uuid,
                                        parent_name: &str| {
                    let repo = repo.clone();
                    let path = path.to_owned();
                    let mut progress_clone = CrawlProgress {
                        files_processed: progress.files_processed,
                        files_indexed: progress.files_indexed,
                        errors: progress.errors.clone(),
                    };
                    let token = token.clone();
                    let parent_name = parent_name.to_owned();
                    let branch_processor = self.branch_processor.clone();

                    Box::pin(async move {
                        let project_start_files = progress_clone.files_processed;
                        branch_processor
                            .process_all_branches_with_tracking(
                                &repo,
                                &path,
                                &mut progress_clone,
                                &token,
                                parent_id,
                                project_start_files,
                                &parent_name,
                            )
                            .await
                    }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>
                };

                let update_crawl_time_fn = |repo_id: Uuid, duration: Option<i32>| {
                    let service = self.clone();
                    Box::pin(async move { service.update_repository_crawl_time(repo_id, duration).await })
                        as std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>
                };

                let cleanup_token_fn = |repo_id: Uuid| {
                    let service = self.clone();
                    Box::pin(async move { service.cleanup_cancellation_token(repo_id).await })
                        as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
                };

                self.gitlab_crawler
                    .crawl_gitlab_repository(
                        repository,
                        cancellation_token,
                        clone_or_update_fn,
                        process_files_fn,
                        update_crawl_time_fn,
                        cleanup_token_fn,
                    )
                    .await?;
            }
            RepositoryType::GitHub => {
                // For GitHub repositories, discover and clone all sub-repositories
                // Create closures for GitHub crawler callbacks
                let clone_or_update_fn = |repo: &Repository, path: &Path| {
                    let repo = repo.clone();
                    let path = path.to_owned();
                    let git_ops = self.git_operations.clone();
                    Box::pin(async move { git_ops.clone_or_update_repository(&repo, &path).await })
                        as std::pin::Pin<Box<dyn std::future::Future<Output = Result<gix::Repository>> + Send>>
                };

                let process_files_fn = |repo: &Repository,
                                        path: &Path,
                                        progress: &mut CrawlProgress,
                                        token: &CancellationToken,
                                        parent_id: Uuid,
                                        parent_name: &str| {
                    let repo = repo.clone();
                    let path = path.to_owned();
                    let mut progress_clone = CrawlProgress {
                        files_processed: progress.files_processed,
                        files_indexed: progress.files_indexed,
                        errors: progress.errors.clone(),
                    };
                    let token = token.clone();
                    let parent_name = parent_name.to_owned();
                    let branch_processor = self.branch_processor.clone();

                    Box::pin(async move {
                        let project_start_files = progress_clone.files_processed;
                        branch_processor
                            .process_all_branches_with_tracking(
                                &repo,
                                &path,
                                &mut progress_clone,
                                &token,
                                parent_id,
                                project_start_files,
                                &parent_name,
                            )
                            .await
                    }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>
                };

                let update_crawl_time_fn = |repo_id: Uuid, duration: Option<i32>| {
                    let service = self.clone();
                    Box::pin(async move { service.update_repository_crawl_time(repo_id, duration).await })
                        as std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>
                };

                let cleanup_token_fn = |repo_id: Uuid| {
                    let service = self.clone();
                    Box::pin(async move { service.cleanup_cancellation_token(repo_id).await })
                        as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
                };

                self.github_crawler
                    .crawl_github_repository(
                        repository,
                        cancellation_token,
                        clone_or_update_fn,
                        process_files_fn,
                        update_crawl_time_fn,
                        cleanup_token_fn,
                    )
                    .await?;
            }
        };

        // Finalize the crawl: commit Tantivy index and update database
        self.finalize_crawl(repository, crawl_start_time).await?;

        info!(
            "Crawl completed for repository: {}. Files processed: {}, Files indexed: {}, Errors: {}",
            repository.name,
            progress.files_processed,
            progress.files_indexed,
            progress.errors.len()
        );

        if !progress.errors.is_empty() {
            warn!("Crawl errors: {:?}", progress.errors);
        }

        Ok(())
    }

    /// Process repository files using the branch processor
    pub async fn process_repository_files(
        &self,
        repository: &Repository,
        repo_path: &Path,
        progress: &mut CrawlProgress,
        cancellation_token: &CancellationToken,
    ) -> Result<()> {
        // Get all branches and process each one
        match self.branch_processor.process_all_branches(repository, repo_path, progress, cancellation_token).await {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!("Failed to process all branches, falling back to default branch: {}", e);
                // Fallback to processing just the current branch
                let branch_name = repository.branch.as_deref().unwrap_or("HEAD");
                self.branch_processor
                    .process_repository_files_internal(
                        repository,
                        repo_path,
                        branch_name,
                        progress,
                        cancellation_token,
                        None,
                        None, // No parent project name for regular Git repos
                    )
                    .await
            }
        }
    }

    /// Check if a file is supported for indexing based on its extension or name
    #[allow(dead_code)]
    pub fn is_supported_file(&self, file_path: &Path) -> bool {
        Self::is_supported_file_static(file_path)
    }

    /// Static version of is_supported_file for use without instance
    pub fn is_supported_file_static(file_path: &Path) -> bool {
        if let Some(extension) = file_path.extension().and_then(|ext| ext.to_str()) {
            SUPPORTED_EXTENSIONS.contains(&extension.to_lowercase().as_str())
        } else {
            // Support files without extensions that might be scripts or config files
            if let Some(file_name) = file_path.file_name().and_then(|name| name.to_str()) {
                matches!(
                    file_name.to_lowercase().as_str(),
                    "dockerfile"
                        | "makefile"
                        | "rakefile"
                        | "gemfile"
                        | "vagrantfile"
                        | "procfile"
                        | "readme"
                        | "license"
                        | "changelog"
                        | "authors"
                        | "contributors"
                        | "copying"
                        | "install"
                        | "news"
                        | "todo"
                )
            } else {
                false
            }
        }
    }

    /// Update repository last_crawled timestamp and duration
    pub async fn update_repository_crawl_time(&self, repository_id: Uuid, duration_seconds: Option<i32>) -> Result<()> {
        let query = if let Some(duration) = duration_seconds {
            sqlx::query("UPDATE repositories SET last_crawled = $1, last_crawl_duration_seconds = $2, updated_at = $1 WHERE id = $3")
                .bind(chrono::Utc::now())
                .bind(duration)
                .bind(repository_id)
        } else {
            sqlx::query("UPDATE repositories SET last_crawled = $1, updated_at = $1 WHERE id = $2")
                .bind(chrono::Utc::now())
                .bind(repository_id)
        };

        query.execute(&self.database).await.map_err(|e| anyhow!("Failed to update repository crawl time: {}", e))?;

        if let Some(duration) = duration_seconds {
            info!(
                "Updated repository {} crawl time with duration: {}s",
                repository_id, duration
            );
        } else {
            info!("Updated repository {} crawl time", repository_id);
        }

        Ok(())
    }

    /// Cancel an ongoing crawl for a repository
    #[allow(dead_code)]
    pub async fn cancel_crawl(&self, repository_id: Uuid) -> Result<bool> {
        let tokens = self.cancellation_tokens.read().await;
        if let Some(token) = tokens.get(&repository_id) {
            token.cancel();
            info!("Cancellation requested for repository: {}", repository_id);
            Ok(true)
        } else {
            warn!("No active crawl found for repository: {}", repository_id);
            Ok(false)
        }
    }

    /// Check if a repository is currently being crawled
    pub async fn is_crawling(&self, repository_id: Uuid) -> bool {
        let tokens = self.cancellation_tokens.read().await;
        tokens.contains_key(&repository_id)
    }

    /// Clean up cancellation token after crawl completion or cancellation
    async fn cleanup_cancellation_token(&self, repository_id: Uuid) {
        let mut tokens = self.cancellation_tokens.write().await;
        tokens.remove(&repository_id);
    }

    /// Check for incomplete crawls and resume them
    pub async fn check_and_resume_incomplete_crawls(&self) -> Result<()> {
        let repo_repo = RepositoryRepository::new(self.database.clone());

        info!("Checking for incomplete crawls to resume...");

        // Find repositories that were being crawled when server crashed
        let incomplete_repos = repo_repo.find_incomplete_crawls().await?;

        if incomplete_repos.is_empty() {
            info!("No incomplete crawls found");
            return Ok(());
        }

        info!("Found {} incomplete crawls to resume", incomplete_repos.len());

        for repository in incomplete_repos {
            info!(
                "Resuming crawl for repository: {} (last project: {:?})",
                repository.name, repository.last_processed_project
            );

            // Resume the crawl from where it left off
            match self.resume_repository_crawl(&repository).await {
                Ok(()) => {
                    info!("Successfully resumed crawl for repository: {}", repository.name);
                }
                Err(e) => {
                    error!("Failed to resume crawl for repository {}: {}", repository.name, e);
                    // Mark as failed so it doesn't get stuck in "in_progress" state
                    let _ = repo_repo.fail_crawl(repository.id).await;
                }
            }
        }

        Ok(())
    }

    /// Resume a repository crawl from where it left off
    pub async fn resume_repository_crawl(&self, repository: &Repository) -> Result<()> {
        let crawl_start_time = std::time::Instant::now();

        info!(
            "Resuming crawl for repository: {} from project: {:?}",
            repository.name, repository.last_processed_project
        );

        match repository.repository_type {
            RepositoryType::GitLab => {
                // Create closures for GitLab crawler callbacks
                let clone_or_update_fn = |repo: &Repository, path: &Path| {
                    let repo = repo.clone();
                    let path = path.to_owned();
                    let git_ops = self.git_operations.clone();
                    Box::pin(async move { git_ops.clone_or_update_repository(&repo, &path).await })
                        as std::pin::Pin<Box<dyn std::future::Future<Output = Result<gix::Repository>> + Send>>
                };

                let process_files_fn = |repo: &Repository,
                                        path: &Path,
                                        progress: &mut CrawlProgress,
                                        token: &CancellationToken,
                                        parent_id: Uuid,
                                        parent_name: &str| {
                    let repo = repo.clone();
                    let path = path.to_owned();
                    let mut progress_clone = CrawlProgress {
                        files_processed: progress.files_processed,
                        files_indexed: progress.files_indexed,
                        errors: progress.errors.clone(),
                    };
                    let token = token.clone();
                    let parent_name = parent_name.to_owned();
                    let branch_processor = self.branch_processor.clone();

                    Box::pin(async move {
                        let project_start_files = progress_clone.files_processed;
                        branch_processor
                            .process_all_branches_with_tracking(
                                &repo,
                                &path,
                                &mut progress_clone,
                                &token,
                                parent_id,
                                project_start_files,
                                &parent_name,
                            )
                            .await
                    }) as std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>
                };

                let update_crawl_time_fn = |repo_id: Uuid, duration: Option<i32>| {
                    let service = self.clone();
                    Box::pin(async move { service.update_repository_crawl_time(repo_id, duration).await })
                        as std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>
                };

                let cleanup_token_fn = |repo_id: Uuid| {
                    let service = self.clone();
                    Box::pin(async move { service.cleanup_cancellation_token(repo_id).await })
                        as std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
                };

                self.gitlab_crawler
                    .resume_gitlab_repository_crawl(
                        repository,
                        clone_or_update_fn,
                        process_files_fn,
                        update_crawl_time_fn,
                        cleanup_token_fn,
                    )
                    .await?;

                // Finalize the crawl: commit Tantivy index and update database
                self.finalize_crawl(repository, crawl_start_time).await
            }
            RepositoryType::Git | RepositoryType::FileSystem | RepositoryType::GitHub => {
                // For Git, FileSystem, and GitHub, just restart the entire crawl
                // since tracking at project level doesn't apply (or not yet implemented for GitHub)
                info!(
                    "Git/FileSystem/GitHub repository, restarting entire crawl: {}",
                    repository.name
                );
                self.crawl_repository(repository).await
            }
        }
    }

    /// Clean up abandoned crawls that have been running for too long
    pub async fn cleanup_abandoned_crawls(&self, timeout_minutes: i64) -> Result<()> {
        let repo_repo = RepositoryRepository::new(self.database.clone());

        info!(
            "Checking for abandoned crawls (timeout: {} minutes)...",
            timeout_minutes
        );

        let abandoned_repos = repo_repo.find_abandoned_crawls(timeout_minutes).await?;

        if abandoned_repos.is_empty() {
            info!("No abandoned crawls found");
            return Ok(());
        }

        info!("Found {} abandoned crawls to clean up", abandoned_repos.len());

        for repository in abandoned_repos {
            warn!(
                "Marking abandoned crawl as failed: {} (started at: {:?})",
                repository.name, repository.crawl_started_at
            );
            repo_repo.fail_crawl(repository.id).await?;
        }

        Ok(())
    }

    /// Finalize a crawl by committing the Tantivy index and updating the database
    /// This is called once at the end of all crawl operations (regular, GitLab, GitHub, resume)
    async fn finalize_crawl(&self, repository: &Repository, crawl_start_time: std::time::Instant) -> Result<()> {
        let repo_repo = RepositoryRepository::new(self.database.clone());

        // Commit the Tantivy index to make changes searchable
        info!("Committing Tantivy index for repository: {}", repository.name);
        tokio::time::timeout(
            std::time::Duration::from_secs(60), // 1 minute timeout for Tantivy commit
            self.search_service.commit(),
        )
        .await
        .map_err(|_| anyhow!("Tantivy commit timed out after 1 minute"))??;

        // Update repository last_crawled timestamp with duration
        let crawl_duration_seconds = crawl_start_time.elapsed().as_secs() as i32;
        self.update_repository_crawl_time(repository.id, Some(crawl_duration_seconds)).await?;

        // Mark crawl as completed in database
        repo_repo.complete_crawl(repository.id).await?;

        // Complete the progress tracking
        self.progress_tracker.complete_crawl(repository.id).await;

        // Clean up cancellation token
        self.cleanup_cancellation_token(repository.id).await;

        info!("Crawl completed for repository: {}", repository.name);

        Ok(())
    }
}

// Implement Clone for CrawlerService to support closure requirements
impl Clone for CrawlerService {
    fn clone(&self) -> Self {
        Self {
            database: self.database.clone(),
            search_service: self.search_service.clone(),
            progress_tracker: self.progress_tracker.clone(),
            encryption_service: self.encryption_service.clone(),
            temp_dir: self.temp_dir.clone(),
            cancellation_tokens: self.cancellation_tokens.clone(),
            git_operations: GitOperations::new(self.encryption_service.clone()),
            branch_processor: BranchProcessor::new(self.search_service.clone(), self.progress_tracker.clone()),
            gitlab_crawler: GitLabCrawler::new(
                self.database.clone(),
                self.search_service.clone(),
                self.progress_tracker.clone(),
                self.encryption_service.clone(),
                self.temp_dir.clone(),
            ),
            github_crawler: GitHubCrawler::new(
                self.database.clone(),
                self.search_service.clone(),
                self.progress_tracker.clone(),
                self.encryption_service.clone(),
                self.temp_dir.clone(),
            ),
        }
    }
}
