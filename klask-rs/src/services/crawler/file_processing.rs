use crate::models::{Repository, RepositoryType};
use crate::services::parser::{PARSER_DISPATCHER, ParsedContent};
use crate::services::search::{FileData, SearchService};
use anyhow::Result;
use mimetype_detector::kind::MimeKind;
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

    /// Check if a file is supported for indexing based on extension or MIME detection
    /// This performs a quick check: extension first, then MIME detection as fallback.
    /// Returns false only for files we're confident are binary or should be skipped.
    pub fn is_supported_file(file_path: &Path) -> bool {
        let extension = file_path.extension().and_then(|ext| ext.to_str());
        let file_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown");

        // Check if any parser supports this extension
        if let Some(ext) = extension {
            let ext_lower = ext.to_lowercase();
            if PARSER_DISPATCHER.all_supported_extensions().iter().any(|e| e.to_lowercase() == ext_lower) {
                debug!("[is_supported_file] {} - ACCEPTED (known extension: {})", file_name, ext);
                return true;
            }
        }

        // Support well-known extensionless files
        if let Some(file_name_str) = file_path.file_name().and_then(|name| name.to_str()) {
            let file_name_lower = file_name_str.to_lowercase();
            if matches!(
                file_name_lower.as_str(),
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
            ) {
                debug!("[is_supported_file] {} - ACCEPTED (well-known name)", file_name);
                return true;
            }
        }

        // For files with unknown extensions, try MIME detection as fallback
        // This allows text files like "_helpers.tpl" to be processed
        if extension.is_some() {
            // Only check MIME for files with extensions, not extensionless files
            if let Ok(bytes) = std::fs::read(file_path) {
                // Read first 8KB for MIME detection (fast, doesn't read entire file)
                let sample_size = std::cmp::min(bytes.len(), 8192);
                let sample = &bytes[..sample_size];

                let mime_type = mimetype_detector::detect(sample);
                let kind = mime_type.kind();

                debug!(
                    "[is_supported_file] {} - unknown extension {:?}, detected MIME: {}, kind: {:?}",
                    file_name,
                    extension,
                    mime_type.mime(),
                    kind
                );

                // Accept if detected as text
                if kind.contains(MimeKind::TEXT) {
                    debug!("[is_supported_file] {} - ACCEPTED (TEXT MIME kind detected)", file_name);
                    return true;
                }

                debug!("[is_supported_file] {} - REJECTED (not TEXT kind)", file_name);
            } else {
                debug!("[is_supported_file] {} - REJECTED (could not read file)", file_name);
            }
        } else {
            debug!("[is_supported_file] {} - REJECTED (no extension, not well-known name)", file_name);
        }

        false
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_mime_detection_for_tpl_template() {
        let tpl_content = b"{{ .Title }}\n{{ range .Items }}\n  <li>{{ . }}</li>\n{{ end }}";
        let mime_type = mimetype_detector::detect(tpl_content);
        let kind = mime_type.kind();

        println!("\n[DEBUG] TPL Template MIME Detection:");
        println!("  MIME: {}", mime_type.mime());
        println!("  Kind: {:?}", kind);
        println!("  Contains TEXT: {}", kind.contains(MimeKind::TEXT));
        println!("  Contains DOCUMENT: {}", kind.contains(MimeKind::DOCUMENT));
        println!("  TEXT contains kind: {}", MimeKind::TEXT.contains(kind));
        println!("  DOCUMENT contains kind: {}", MimeKind::DOCUMENT.contains(kind));
    }

    #[test]
    fn test_is_supported_file_with_tpl_extension() {
        // Create a temporary .tpl file
        let test_file = "/tmp/test_helpers.tpl";
        let tpl_content = "{{ .Title }}\n{{ range .Items }}\n  <li>{{ . }}</li>\n{{ end }}";
        fs::write(test_file, tpl_content).unwrap();

        let path = Path::new(test_file);

        // Also test the MIME detection directly
        let bytes = fs::read(test_file).unwrap();
        let mime_type = mimetype_detector::detect(&bytes);
        println!("\n[TEST] _helpers.tpl file:");
        println!("  MIME: {}", mime_type.mime());
        println!("  Kind: {:?}", mime_type.kind());

        let result = FileProcessor::is_supported_file(path);

        println!("  is_supported_file result: {}", result);

        // Clean up
        let _ = fs::remove_file(test_file);

        assert!(result, "File with .tpl extension should be supported as TEXT");
    }

    #[test]
    fn test_is_supported_file_with_real_helpers_tpl() {
        // Use the actual content from klask's _helpers.tpl
        let real_tpl_content = r#"{{/*
Expand the name of the chart.
*/}}
{{- define "klask.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create chart name and version
*/}}
{{- define "klask.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}"#;

        let test_file = "/tmp/test_real_helpers.tpl";
        fs::write(test_file, real_tpl_content).unwrap();

        let path = Path::new(test_file);

        // Check MIME detection
        let bytes = fs::read(test_file).unwrap();
        let mime_type = mimetype_detector::detect(&bytes);
        println!("\n[TEST] Real _helpers.tpl file:");
        println!("  MIME: {}", mime_type.mime());
        println!("  Kind: {:?}", mime_type.kind());
        println!("  Contains TEXT: {}", mime_type.kind().contains(MimeKind::TEXT));

        let result = FileProcessor::is_supported_file(path);
        println!("  is_supported_file result: {}", result);

        // Clean up
        let _ = fs::remove_file(test_file);

        assert!(result, "Real _helpers.tpl should be supported");
    }

    #[test]
    fn test_is_supported_file_with_known_extension() {
        let test_file = "/tmp/test_file.rs";
        fs::write(test_file, "fn main() {}").unwrap();

        let path = Path::new(test_file);
        let result = FileProcessor::is_supported_file(path);

        println!("[TEST] is_supported_file for .rs file: {}", result);

        // Clean up
        let _ = fs::remove_file(test_file);

        assert!(result, "File with .rs extension should be supported");
    }
}
