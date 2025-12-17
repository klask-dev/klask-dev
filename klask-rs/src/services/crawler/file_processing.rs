use crate::models::{Repository, RepositoryType};
use crate::services::parser::{PARSER_DISPATCHER, ParsedContent};
use crate::services::search::{FileData, SearchService};
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, error};
use uuid::Uuid;

/// File processing utilities for the crawler
#[derive(Clone)]
pub struct FileProcessor {
    search_service: Arc<SearchService>,
}

impl FileProcessor {
    pub fn new(search_service: Arc<SearchService>) -> Self {
        Self { search_service }
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
    /// If `provided_content` is Some, it will be used directly instead of reading and parsing from disk.
    /// This is useful when reading from Git trees without checking out files.
    pub async fn process_single_file(
        &self,
        repository: &Repository,
        file_path: &Path,
        relative_path: &str,
        branch_name: &str,
        parent_project_name: Option<&str>,
        provided_content: Option<ParsedContent>,
    ) -> Result<()> {
        // Get parsed content - use provided content if available, otherwise read and parse from disk
        let parsed_content = if let Some(content) = provided_content {
            debug!(
                "[GIT READ] Processing file {} in branch '{}' from Git (provided content: {} bytes)",
                relative_path,
                branch_name,
                content.text.len()
            );
            Some(content)
        } else {
            debug!(
                "[DISK READ] Reading file {} in branch '{}' from filesystem",
                relative_path, branch_name
            );

            // Read and parse from disk
            match tokio::fs::read(file_path).await {
                Ok(bytes) => {
                    let extension = file_path.extension().and_then(|e| e.to_str());

                    match PARSER_DISPATCHER.parse(&bytes, relative_path, extension) {
                        Ok(parsed) => {
                            debug!(
                                "[DISK READ] Successfully parsed file {} ({} bytes -> {} chars)",
                                relative_path,
                                bytes.len(),
                                parsed.text.len()
                            );
                            Some(parsed)
                        }
                        Err(e) => {
                            debug!("[DISK READ] Skipping file {}: {}", relative_path, e);
                            None
                        }
                    }
                }
                Err(e) => {
                    debug!("[DISK READ] Could not read file: {} - Error: {}", relative_path, e);
                    None
                }
            }
        };

        let extension = file_path.extension().and_then(|ext| ext.to_str()).unwrap_or("").to_string();

        let file_name = file_path.file_name().and_then(|name| name.to_str()).unwrap_or("").to_string();

        // Index in Tantivy search engine if parsed content is available
        if let Some(parsed) = parsed_content {
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
                    content: &parsed.text,
                    repository: repository_field, // Parent repository for mass deletion
                    project: &repository.name,    // Individual project name for facets
                    version: &version,
                    extension: &extension,
                    size: parsed.text.len() as u64, // Calculate size from content length
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
        }

        Ok(())
    }

    /// Check if a file is supported for indexing based on extension
    /// Note: This does a lightweight check based on extension only.
    /// Actual MIME detection happens during parsing.
    pub fn is_supported_file(file_path: &Path) -> bool {
        let extension = file_path.extension().and_then(|ext| ext.to_str());

        // Check if any parser supports this extension
        if let Some(ext) = extension {
            let ext_lower = ext.to_lowercase();
            return PARSER_DISPATCHER.all_supported_extensions().iter().any(|e| e.to_lowercase() == ext_lower);
        }

        // Support well-known extensionless files
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

    /// Collect all supported files from a directory recursively
    #[allow(dead_code)]
    pub fn collect_supported_files(repo_path: &Path) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();
        Self::collect_files_recursive(repo_path, &mut files)?;
        Ok(files.into_iter().filter(|path| Self::is_supported_file(path)).collect())
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
