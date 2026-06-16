use crate::models::{Repository, RepositoryType};
use crate::services::crawler::parsers::ParserDispatcher;
use crate::services::search::{FileData, SearchService};
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
#[cfg(feature = "semantic-search")]
use tracing::warn;
use tracing::{debug, error};
use uuid::Uuid;

/// File processing utilities for the crawler
#[derive(Clone)]
pub struct FileProcessor {
    search_service: Arc<SearchService>,
    /// Optional semantic indexer: mirrors each indexed file into the vector
    /// store. None when semantic search is disabled / not compiled in. Read only
    /// in the `semantic-search` build.
    #[cfg_attr(not(feature = "semantic-search"), allow(dead_code))]
    semantic_indexer: crate::services::semantic::MaybeIndexer,
}

impl FileProcessor {
    pub fn new(search_service: Arc<SearchService>, semantic_indexer: crate::services::semantic::MaybeIndexer) -> Self {
        Self { search_service, semantic_indexer }
    }

    /// Generate a deterministic UUID for a file based on repository, specific branch, and path
    pub fn generate_deterministic_file_id(repository: &Repository, relative_path: &str, branch_name: &str) -> Uuid {
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

        debug!(
            "Generating file ID from input: '{}' for file {} in branch {}",
            input, relative_path, branch_name
        );

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

    /// Process a single file and index it in the search service
    ///
    /// If `provided_content` is Some, it will be used directly instead of reading from disk.
    /// This is useful when reading from Git trees without checking out files.
    pub async fn process_single_file(
        &self,
        repository: &Repository,
        file_path: &Path,
        relative_path: &str,
        branch_name: &str,
        parent_project_name: Option<&str>,
        provided_content: Option<String>,
    ) -> Result<()> {
        let extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("").to_string();
        let file_name = file_path.file_name().and_then(|name| name.to_str()).unwrap_or("").to_string();

        // Check if file should be filtered (only binary files are filtered)
        if ParserDispatcher::should_filter(&extension) {
            debug!("Skipping binary file (extension {}): {}", extension, relative_path);
            return Ok(());
        }

        // Get the appropriate parser for this file extension
        let parser = ParserDispatcher::get_parser(&extension);

        // Read file content - use provided content if available, otherwise read from disk
        let raw_content = if let Some(content) = provided_content {
            debug!(
                "[GIT READ] Processing file {} in branch '{}' from Git (provided content: {} bytes)",
                relative_path,
                branch_name,
                content.len()
            );
            content.into_bytes()
        } else {
            debug!(
                "[DISK READ] Reading file {} in branch '{}' from filesystem",
                relative_path, branch_name
            );

            match tokio::fs::read(file_path).await {
                Ok(bytes) => {
                    debug!(
                        "[DISK READ] Successfully read file {} ({} bytes)",
                        relative_path,
                        bytes.len()
                    );
                    bytes
                }
                Err(e) => {
                    debug!("[DISK READ] Could not read file: {} - Error: {}", relative_path, e);
                    return Ok(());
                }
            }
        };

        // Parse the file content
        // For empty files, use empty string directly (still index for filename)
        let content = if raw_content.is_empty() {
            debug!("File {} is empty, will be indexed with filename only", relative_path);
            String::new()
        } else {
            match parser.parse(&raw_content, Some(relative_path)).await {
                Ok(parsed_content) => parsed_content,
                Err(e) => {
                    debug!(
                        "Failed to parse file {} (extension {}): {}",
                        relative_path, extension, e
                    );
                    return Ok(());
                }
            }
        };

        // Generate a deterministic ID for Tantivy indexing to prevent duplicates
        let file_id = Self::generate_deterministic_file_id(repository, relative_path, branch_name);
        let version = branch_name.to_string();

        // For repository: use parent project name if provided (for GitLab/GitHub multi-project repos),
        // otherwise use repository name (for regular Git repos)
        let repository_field = parent_project_name.unwrap_or(&repository.name);

        debug!(
            "Indexing file {} with deterministic ID {} for branch '{}' - repository: {}, project: {}",
            relative_path, file_id, branch_name, repository_field, repository.name
        );

        // Use upsert to handle potential duplicates - this will update existing docs
        match self
            .search_service
            .upsert_file(FileData {
                file_id,
                file_name: &file_name,
                file_path: relative_path,
                content: &content,
                repository: repository_field, // Parent repository for mass deletion
                project: &repository.name,    // Individual project name for facets
                version: &version,
                extension: &extension,
                size: raw_content.len() as u64, // Use raw file size (before parsing)
            })
            .await
        {
            Ok(_) => {
                debug!(
                    "Successfully upserted file {} to Tantivy index for branch '{}'",
                    relative_path, branch_name
                );
            }
            Err(e) => {
                error!(
                    "Failed to upsert file {} to Tantivy index for branch '{}': {}",
                    relative_path, branch_name, e
                );
                return Err(e);
            }
        }

        // Mirror the file into the semantic vector store (best-effort). Tantivy
        // is the source of truth; a failed/blocked embed must not fail the
        // crawl — Phase 3 backfill reconciles any gap. With strict backpressure
        // this `await` may pause the crawl until the embedding queue drains.
        #[cfg(feature = "semantic-search")]
        if let Some(indexer) = &self.semantic_indexer {
            let job = crate::services::semantic::IndexJob {
                file_id,
                repository: repository_field.to_string(),
                project: repository.name.clone(),
                version: version.clone(),
                path: relative_path.to_string(),
                extension: extension.clone(),
                content: content.clone(),
            };
            if let Err(e) = indexer.index_file(job).await {
                warn!("Failed to enqueue {} for semantic indexing: {}", relative_path, e);
            }
        }

        Ok(())
    }

    /// Recursively collect files from a directory
    #[allow(dead_code)]
    fn collect_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Skip hidden directories and common ignore patterns
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str())
                    && (dir_name.starts_with('.')
                        || dir_name == "node_modules"
                        || dir_name == "target"
                        || dir_name == "__pycache__")
                {
                    continue;
                }
                Self::collect_files_recursive(&path, files)?;
            } else if path.is_file() {
                files.push(path);
            }
        }
        Ok(())
    }
}
