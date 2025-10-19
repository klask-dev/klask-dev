use super::file_processing::FileProcessor;
use super::git_tree_walker::GitTreeWalker;
use crate::models::Repository;
use crate::services::progress::ProgressTracker;
use crate::services::search::SearchService;
use anyhow::{anyhow, Result};
use gix::ObjectId;
use std::path::Path;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Represents a file entry in a Git tree
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GitFileEntry {
    pub path: String,
    pub oid: ObjectId,
}

/// Progress tracking for crawl operations
pub struct CrawlProgress {
    pub files_processed: usize,
    pub files_indexed: usize,
    pub errors: Vec<String>,
}

/// Branch processing operations for the crawler
#[derive(Clone)]
pub struct BranchProcessor {
    search_service: Arc<SearchService>,
    progress_tracker: Arc<ProgressTracker>,
    file_processor: FileProcessor,
}

impl BranchProcessor {
    pub fn new(search_service: Arc<SearchService>, progress_tracker: Arc<ProgressTracker>) -> Self {
        let file_processor = FileProcessor::new(search_service.clone());
        Self { search_service, progress_tracker, file_processor }
    }

    /// Process all branches in a repository
    pub async fn process_all_branches(
        &self,
        repository: &Repository,
        repo_path: &Path,
        progress: &mut CrawlProgress,
        cancellation_token: &CancellationToken,
    ) -> Result<()> {
        info!("Discovering branches for repository: {}", repository.name);

        // Get all branches using gix
        let repo_path_owned = repo_path.to_owned();
        let branches = tokio::task::spawn_blocking(move || -> Result<Vec<String>> {
            let git_repo = gix::open(&repo_path_owned)?;
            GitTreeWalker::get_all_branches(&git_repo)
        })
        .await??;

        if branches.is_empty() {
            info!("No branches found, using default branch");
            let branch_name = repository.branch.as_deref().unwrap_or("main");
            return self
                .process_repository_files_internal(
                    repository,
                    repo_path,
                    branch_name,
                    progress,
                    cancellation_token,
                    None,
                    None, // No parent project name for regular Git repos
                )
                .await;
        }

        info!(
            "Found {} branches for repository {}: {:?}",
            branches.len(),
            repository.name,
            branches
        );

        // Process each branch
        for branch_name in branches {
            if cancellation_token.is_cancelled() {
                return Ok(());
            }

            info!("Processing branch '{}' for repository {}", branch_name, repository.name);

            // Process files from this branch's Git tree
            match self
                .process_branch_from_tree(
                    repository,
                    repo_path,
                    &branch_name,
                    progress,
                    cancellation_token,
                    None,
                    None, // No parent project name for regular Git repos
                )
                .await
            {
                Ok(()) => {
                    info!(
                        "Successfully processed branch '{}' for repository {}",
                        branch_name, repository.name
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to process branch '{}' for repository {}: {}",
                        branch_name, repository.name, e
                    );
                    progress.errors.push(format!("Branch '{}': {}", branch_name, e));
                }
            }
        }

        Ok(())
    }

    /// Process all branches with GitLab/GitHub tracking
    #[allow(clippy::too_many_arguments)]
    pub async fn process_all_branches_with_tracking(
        &self,
        repository: &Repository,
        repo_path: &Path,
        progress: &mut CrawlProgress,
        cancellation_token: &CancellationToken,
        parent_repository_id: Uuid,
        _project_start_files: usize,
        parent_project_name: &str, // Parent repository name for GitLab/GitHub multi-project repos
    ) -> Result<()> {
        // Just call the normal process_all_branches but pass through the tracking info
        // to each branch processing call
        info!("Discovering branches for repository: {}", repository.name);

        // Get all branches using gix
        let repo_path_owned = repo_path.to_owned();
        let branches = tokio::task::spawn_blocking(move || -> Result<Vec<String>> {
            let git_repo = gix::open(&repo_path_owned)?;
            GitTreeWalker::get_all_branches(&git_repo)
        })
        .await??;

        if branches.is_empty() {
            info!("No branches found, using default branch");
            let branch_name = repository.branch.as_deref().unwrap_or("main");
            return self
                .process_repository_files_internal(
                    repository,
                    repo_path,
                    branch_name,
                    progress,
                    cancellation_token,
                    Some((parent_repository_id, _project_start_files)),
                    Some(parent_project_name),
                )
                .await;
        }

        info!(
            "Found {} branches for repository {}: {:?}",
            branches.len(),
            repository.name,
            branches
        );

        // Process each branch
        for branch_name in branches {
            if cancellation_token.is_cancelled() {
                return Ok(());
            }

            info!("Processing branch '{}' for repository {}", branch_name, repository.name);

            // Process files from this branch's Git tree with tracking
            match self
                .process_branch_from_tree(
                    repository,
                    repo_path,
                    &branch_name,
                    progress,
                    cancellation_token,
                    Some(parent_repository_id),
                    Some(parent_project_name), // Pass parent project name for GitLab/GitHub
                )
                .await
            {
                Ok(()) => {
                    info!(
                        "Successfully processed branch '{}' for repository {}",
                        branch_name, repository.name
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to process branch '{}' for repository {}: {}",
                        branch_name, repository.name, e
                    );
                    progress.errors.push(format!("Branch '{}': {}", branch_name, e));
                }
            }
        }

        Ok(())
    }

    /// Process files from a branch by reading directly from the Git tree (no checkout needed)
    #[allow(clippy::too_many_arguments)]
    pub async fn process_branch_from_tree(
        &self,
        repository: &Repository,
        repo_path: &Path,
        branch_name: &str,
        progress: &mut CrawlProgress,
        cancellation_token: &CancellationToken,
        parent_repository_id: Option<Uuid>,
        parent_project_name: Option<&str>, // Parent repository name for GitLab/GitHub multi-project repos
    ) -> Result<()> {
        use super::git_tree_walker::{GitFileEntry, MAX_FILE_SIZE};

        let repo_path_owned = repo_path.to_owned();
        let branch_name_owned = branch_name.to_string();

        // Get tree ID and files from the Git database
        let files = tokio::task::spawn_blocking(move || -> Result<Vec<GitFileEntry>> {
            let git_repo = gix::open(&repo_path_owned)?;

            // Get the tree ID for this branch
            let tree_id = GitTreeWalker::get_branch_tree_id(&git_repo, &branch_name_owned)?;

            // Walk the tree and collect all files
            let files = GitTreeWalker::walk_tree(&git_repo, &tree_id, "")?;

            info!("Found {} files in branch '{}'", files.len(), branch_name_owned);
            Ok(files)
        })
        .await??;

        // Update progress tracking if parent_repository_id is provided
        if let Some(parent_id) = parent_repository_id {
            let project_with_branch = format!("{} ({})", repository.name, branch_name);
            self.progress_tracker.set_current_gitlab_project(parent_id, Some(project_with_branch)).await;

            // Filter to supported files for progress tracking
            let supported_files: Vec<&GitFileEntry> =
                files.iter().filter(|f| Self::is_supported_file_static(std::path::Path::new(&f.path))).collect();

            self.progress_tracker.set_current_project_files_total(parent_id, supported_files.len()).await;
        }

        let total_files = files.len();
        let repo_path_owned = repo_path.to_owned();

        info!(
            "Starting to process {} files for branch '{}' in repository {}",
            total_files, branch_name, repository.name
        );

        let mut files_skipped_by_filter = 0;
        let mut files_read_success = 0;
        let mut files_read_failed = 0;
        let mut files_binary_skipped = 0;

        // Process each file by reading directly from Git
        for (idx, file_entry) in files.iter().enumerate() {
            if idx % 100 == 0 && idx > 0 {
                debug!(
                    "Progress: {}/{} files in branch '{}' - indexed: {}, skipped_filter: {}, binary/too_large: {}, failed: {}",
                    idx, total_files, branch_name, files_read_success, files_skipped_by_filter, files_binary_skipped, files_read_failed
                );
            }
            // Check for cancellation
            if cancellation_token.is_cancelled() {
                info!("Crawl cancelled for repository: {}", repository.name);
                return Ok(());
            }

            // Check if file is supported - skip unsupported files
            if !Self::is_supported_file_static(std::path::Path::new(&file_entry.path)) {
                files_skipped_by_filter += 1;
                continue;
            }

            // Read file content from Git database
            let repo_path_for_task = repo_path_owned.clone();
            let oid = file_entry.oid;
            let path = file_entry.path.clone();

            let content_result = tokio::task::spawn_blocking(move || -> Result<Option<String>> {
                let git_repo = gix::open(&repo_path_for_task)?;

                // Check file size first
                if !GitTreeWalker::check_blob_size(&git_repo, &oid)? {
                    debug!("[GIT] Skipping large file: {} (> {} bytes)", path, MAX_FILE_SIZE);
                    return Ok(None);
                }

                // Read the content
                let result = GitTreeWalker::read_blob_content(&git_repo, &oid);
                match &result {
                    Ok(Some(content)) => {
                        debug!(
                            "[GIT] Successfully read blob for file {} ({} bytes)",
                            path,
                            content.len()
                        );
                    }
                    Ok(None) => {
                        debug!("[GIT] Blob returned None (likely binary) for file {}", path);
                    }
                    Err(e) => {
                        debug!("[GIT] Failed to read blob for file {}: {}", path, e);
                    }
                }
                result
            })
            .await;

            match content_result {
                Ok(Ok(Some(content))) => {
                    // Index the file - pass the content we already read from Git
                    let file_path = std::path::PathBuf::from(&file_entry.path);
                    match self
                        .file_processor
                        .process_single_file(
                            repository,
                            &file_path,
                            &file_entry.path,
                            branch_name,
                            parent_project_name,
                            Some(content), // Pass the content we already have
                        )
                        .await
                    {
                        Ok(()) => {
                            progress.files_indexed += 1;
                            files_read_success += 1;
                            debug!(
                                "[GIT] Successfully indexed file {} in branch '{}'",
                                file_entry.path, branch_name
                            );
                        }
                        Err(e) => {
                            files_read_failed += 1;
                            warn!("[GIT] Failed to index file {}: {}", file_entry.path, e);
                            progress.errors.push(format!("Failed to index {}: {}", file_entry.path, e));
                        }
                    }

                    progress.files_processed += 1;
                }
                Ok(Ok(None)) => {
                    // Skipped (binary or too large)
                    files_binary_skipped += 1;
                    debug!("[GIT] Skipped file {} (binary or too large)", file_entry.path);
                }
                Ok(Err(e)) => {
                    files_read_failed += 1;
                    warn!("[GIT] Failed to read file {}: {}", file_entry.path, e);
                    progress.errors.push(format!("Failed to read {}: {}", file_entry.path, e));
                }
                Err(e) => {
                    files_read_failed += 1;
                    warn!("[GIT] Failed to spawn task for file {}: {}", file_entry.path, e);
                    progress.errors.push(format!("Failed to process {}: {}", file_entry.path, e));
                }
            }
        }

        info!(
            "Completed branch '{}': total={}, indexed={}, skipped_filter={}, binary/too_large={}, failed={}",
            branch_name,
            total_files,
            files_read_success,
            files_skipped_by_filter,
            files_binary_skipped,
            files_read_failed
        );

        // Note: Tantivy commit is now done once at the end of the entire crawl in crawler_service
        // (not after each branch) for better performance
        Ok(())
    }

    /// Process repository files using file system walk (fallback method)
    #[allow(clippy::too_many_arguments)]
    pub async fn process_repository_files_internal(
        &self,
        repository: &Repository,
        repo_path: &Path,
        branch_name: &str,
        progress: &mut CrawlProgress,
        cancellation_token: &CancellationToken,
        gitlab_tracking: Option<(Uuid, usize)>, // (parent_id, _project_start_files_count)
        parent_project_name: Option<&str>,      // Parent repository name for GitLab/GitHub multi-project repos
    ) -> Result<()> {
        use super::git_tree_walker::MAX_FILE_SIZE;
        use walkdir::WalkDir;

        // For Tantivy-only indexing, we don't need to track file deletions
        // since Tantivy will be rebuilt fresh for each crawl

        let repo_path_owned = repo_path.to_owned();
        let repo_path_owned2 = repo_path.to_owned();

        // First pass: Count total eligible files for accurate progress reporting (in blocking thread)
        let total_files = tokio::task::spawn_blocking(move || -> Result<usize> {
            let mut total_files = 0;
            for entry in
                WalkDir::new(&repo_path_owned).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file())
            {
                let file_path = entry.path();
                let relative_path = file_path
                    .strip_prefix(&repo_path_owned)
                    .map_err(|e| anyhow!("Failed to get relative path: {}", e))?;

                let relative_path_str = relative_path.to_string_lossy().to_string();

                // Skip hidden files and directories
                if relative_path_str.starts_with('.') {
                    continue;
                }

                // Check file extension
                if !Self::is_supported_file_static(file_path) {
                    continue;
                }

                // Check file size
                if let Ok(metadata) = file_path.metadata() {
                    if metadata.len() > MAX_FILE_SIZE {
                        continue;
                    }
                }

                total_files += 1;
            }
            Ok(total_files)
        })
        .await??;

        info!(
            "Found {} eligible files to process for branch '{}' in repository {}",
            total_files, branch_name, repository.name
        );

        // Collect all file paths to process (in blocking thread)
        let files_to_process = tokio::task::spawn_blocking(move || -> Result<Vec<(std::path::PathBuf, String)>> {
            let mut files = Vec::new();
            for entry in
                WalkDir::new(&repo_path_owned2).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file())
            {
                let file_path = entry.path();
                let relative_path = file_path
                    .strip_prefix(&repo_path_owned2)
                    .map_err(|e| anyhow!("Failed to get relative path: {}", e))?;

                let relative_path_str = relative_path.to_string_lossy().to_string();

                // Skip hidden files and directories
                if relative_path_str.starts_with('.') {
                    continue;
                }

                // Check file extension
                if !Self::is_supported_file_static(file_path) {
                    continue;
                }

                // Check file size
                if let Ok(metadata) = file_path.metadata() {
                    if metadata.len() > MAX_FILE_SIZE {
                        debug!("Skipping large file: {} ({} bytes)", relative_path_str, metadata.len());
                        continue;
                    }
                }

                files.push((file_path.to_path_buf(), relative_path_str));
            }
            Ok(files)
        })
        .await??;

        info!(
            "Collected {} files to process for branch '{}' in repository {}",
            files_to_process.len(),
            branch_name,
            repository.name
        );

        // Now process all files asynchronously
        info!(
            "Starting to process {} files for branch '{}' in repository {}",
            files_to_process.len(),
            branch_name,
            repository.name
        );
        for (file_path, relative_path_str) in files_to_process {
            // Check for cancellation at the start of each file processing
            if cancellation_token.is_cancelled() {
                info!("Crawl cancelled for repository: {}", repository.name);
                return Ok(());
            }

            progress.files_processed += 1;

            // Update current file being processed
            self.progress_tracker.set_current_file(repository.id, Some(relative_path_str.clone())).await;
            self.progress_tracker
                .update_progress(
                    repository.id,
                    progress.files_processed,
                    Some(total_files),
                    progress.files_indexed,
                )
                .await;

            // Update GitLab project files progress if applicable
            if let Some((parent_id, project_start)) = gitlab_tracking {
                let project_files_processed = progress.files_processed - project_start;
                self.progress_tracker.update_current_project_files(parent_id, project_files_processed).await;
            }

            match self
                .file_processor
                .process_single_file(
                    repository,
                    &file_path,
                    &relative_path_str,
                    branch_name,
                    parent_project_name,
                    None, // No content provided - will read from disk
                )
                .await
            {
                Ok(()) => {
                    progress.files_indexed += 1;
                    debug!(
                        "Successfully indexed file {} in branch '{}' for repository {}",
                        relative_path_str, branch_name, repository.name
                    );
                    // Update indexed count
                    self.progress_tracker
                        .update_progress(
                            repository.id,
                            progress.files_processed,
                            Some(total_files),
                            progress.files_indexed,
                        )
                        .await;

                    // Update GitLab project files progress if applicable (files indexed in current project)
                    if let Some((parent_id, project_start)) = gitlab_tracking {
                        let project_files_processed = progress.files_processed - project_start;
                        self.progress_tracker.update_current_project_files(parent_id, project_files_processed).await;
                    }
                }
                Err(e) => {
                    let error_msg = format!("Failed to process file {}: {}", relative_path_str, e);
                    progress.errors.push(error_msg);
                    error!("Error processing file {}: {}", relative_path_str, e);
                }
            }
        }

        // Clear current file when done
        self.progress_tracker.set_current_file(repository.id, None).await;

        info!(
            "Finished processing {} files for branch '{}' in repository {} - indexed: {}, errors: {}",
            progress.files_processed,
            branch_name,
            repository.name,
            progress.files_indexed,
            progress.errors.len()
        );

        // Note: Tantivy commit is now done once at the end of the entire crawl in crawler_service
        // (not after each branch) for better performance
        Ok(())
    }

    /// Check if a file is supported for indexing based on its extension or name
    fn is_supported_file_static(file_path: &Path) -> bool {
        use super::file_processing::SUPPORTED_EXTENSIONS;

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
}
