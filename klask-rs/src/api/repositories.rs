use crate::auth::extractors::{AdminUser, AppState};
use crate::models::{Repository, RepositoryType};
use crate::repositories::RepositoryRepository;
use crate::services::github::{GitHubRepository, GitHubService};
use crate::services::gitlab::{GitLabProject, GitLabService};
use anyhow::Result;
use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRepositoryRequest {
    pub name: String,
    pub url: String,
    pub repository_type: RepositoryType,
    pub branch: Option<String>,
    pub enabled: Option<bool>,
    pub access_token: Option<String>,
    pub gitlab_namespace: Option<String>,
    pub is_group: Option<bool>,
    // Scheduling fields
    pub auto_crawl_enabled: Option<bool>,
    pub cron_schedule: Option<String>,
    pub crawl_frequency_hours: Option<i32>,
    pub max_crawl_duration_minutes: Option<i32>,
    // GitLab exclusion fields
    pub gitlab_excluded_projects: Option<String>,
    pub gitlab_excluded_patterns: Option<String>,
    // GitHub fields
    pub github_namespace: Option<String>,
    pub github_excluded_repositories: Option<String>,
    pub github_excluded_patterns: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRepositoryRequest {
    pub name: Option<String>,
    pub url: Option<String>,
    pub repository_type: Option<RepositoryType>,
    pub branch: Option<String>,
    pub enabled: Option<bool>,
    pub access_token: Option<String>,
    pub gitlab_namespace: Option<String>,
    pub is_group: Option<bool>,
    // Scheduling fields
    pub auto_crawl_enabled: Option<bool>,
    pub cron_schedule: Option<String>,
    pub crawl_frequency_hours: Option<i32>,
    pub max_crawl_duration_minutes: Option<i32>,
    // GitLab exclusion fields
    pub gitlab_excluded_projects: Option<String>,
    pub gitlab_excluded_patterns: Option<String>,
    // GitHub fields
    pub github_namespace: Option<String>,
    pub github_excluded_repositories: Option<String>,
    pub github_excluded_patterns: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryWithStats {
    #[serde(flatten)]
    pub repository: Repository,
    pub disk_size_mb: Option<f64>,
    pub file_count: Option<i64>,
    pub last_crawl_duration_minutes: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoriesResponse {
    pub repositories: Vec<RepositoryWithStats>,
    pub total: usize,
}

// GitHub API request/response structures
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverGitHubRequest {
    pub access_token: String,
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestGitHubTokenRequest {
    pub access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverGitHubResponse {
    pub repositories: Vec<GitHubRepository>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestTokenResponse {
    pub valid: bool,
    pub message: String,
}

// GitLab API request/response structures
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverGitLabRequest {
    pub gitlab_url: String,
    pub access_token: String,
    pub namespace: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestGitLabTokenRequest {
    pub gitlab_url: String,
    pub access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoverGitLabResponse {
    pub projects: Vec<GitLabProject>,
}

/// Validates GitHub namespace format
/// GitHub namespaces (users/organizations) can only contain:
/// - Alphanumeric characters (a-z, A-Z, 0-9)
/// - Hyphens (-)
/// - Underscores (_)
fn validate_github_namespace(namespace: &str) -> Result<(), String> {
    // Empty namespace is valid (optional field)
    if namespace.is_empty() {
        return Ok(());
    }

    // GitHub namespace validation: alphanumerics, hyphens, and underscores only
    let re = Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
    if !re.is_match(namespace) {
        return Err(
            "Invalid GitHub namespace format. Only alphanumeric characters, hyphens, and underscores are allowed."
                .to_string(),
        );
    }
    Ok(())
}

pub async fn create_router() -> Result<Router<AppState>> {
    let router = Router::new()
        .route("/", get(list_repositories).post(create_repository))
        .route(
            "/{id}",
            get(get_repository).put(update_repository).delete(delete_repository),
        )
        .route("/{id}/crawl", post(crawl_repository).delete(stop_crawl_repository))
        .route("/{id}/test", post(test_repository_connection))
        .route("/{id}/stats", get(get_repository_stats))
        .route("/import/gitlab", post(import_gitlab_projects))
        .route("/github/discover", post(discover_github_repositories))
        .route("/github/test-token", post(test_github_token))
        .route("/gitlab/discover", post(discover_gitlab_repositories))
        .route("/gitlab/test-token", post(test_gitlab_token))
        .route("/bulk/enable", post(bulk_enable_repositories))
        .route("/bulk/disable", post(bulk_disable_repositories))
        .route("/bulk/crawl", post(bulk_crawl_repositories))
        .route("/bulk/delete", delete(bulk_delete_repositories))
        .route("/progress/active", get(get_active_crawl_progress))
        .route("/stats", get(get_repositories_stats));

    Ok(router)
}

async fn list_repositories(
    _user: AdminUser,
    State(app_state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<RepositoriesResponse>, StatusCode> {
    let repo_repository = RepositoryRepository::new(app_state.database.pool().clone());

    match repo_repository.list_repositories().await {
        Ok(repositories) => {
            // Check if stats are requested
            let include_stats = params.get("include_stats").map(|v| v == "true").unwrap_or(false);

            let mut repositories_with_stats = Vec::new();

            if include_stats {
                // Process stats in parallel using futures::future::join_all
                use futures::future::join_all;

                let stat_futures: Vec<_> = repositories
                    .into_iter()
                    .map(|repo| {
                        let app_state = app_state.clone();
                        async move {
                            let disk_size_mb = calculate_repository_disk_size(&repo).await.unwrap_or(0.0);
                            let file_count = get_repository_file_count(&repo, &app_state).await.unwrap_or(0);
                            let last_crawl_duration_minutes =
                                repo.last_crawl_duration_seconds.map(|seconds| seconds as f64 / 60.0);

                            RepositoryWithStats {
                                repository: repo,
                                disk_size_mb: Some(disk_size_mb),
                                file_count: Some(file_count),
                                last_crawl_duration_minutes,
                            }
                        }
                    })
                    .collect();

                repositories_with_stats = join_all(stat_futures).await;
            } else {
                // Fast path: return repositories without expensive stats
                for repo in repositories {
                    let last_crawl_duration_minutes =
                        repo.last_crawl_duration_seconds.map(|seconds| seconds as f64 / 60.0);

                    repositories_with_stats.push(RepositoryWithStats {
                        repository: repo,
                        disk_size_mb: None, // Lazy load these stats
                        file_count: None,   // Lazy load these stats
                        last_crawl_duration_minutes,
                    });
                }
            }

            let total = repositories_with_stats.len();

            Ok(Json(RepositoriesResponse {
                repositories: repositories_with_stats,
                total,
            }))
        }
        Err(e) => {
            error!("Failed to list repositories: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_repository_stats(
    _user: AdminUser,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RepositoryWithStats>, StatusCode> {
    let repo_repository = RepositoryRepository::new(app_state.database.pool().clone());

    match repo_repository.get_repository(id).await {
        Ok(Some(repo)) => {
            let disk_size_mb = calculate_repository_disk_size(&repo).await.unwrap_or(0.0);
            let file_count = get_repository_file_count(&repo, &app_state).await.unwrap_or(0);
            let last_crawl_duration_minutes = repo.last_crawl_duration_seconds.map(|seconds| seconds as f64 / 60.0);

            Ok(Json(RepositoryWithStats {
                repository: repo,
                disk_size_mb: Some(disk_size_mb),
                file_count: Some(file_count),
                last_crawl_duration_minutes,
            }))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get repository stats: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Helper function to calculate repository disk size
async fn calculate_repository_disk_size(repository: &Repository) -> Result<f64> {
    match repository.repository_type {
        RepositoryType::FileSystem => {
            // For filesystem repos, calculate actual directory size
            let path = PathBuf::from(&repository.url);
            if path.exists() && path.is_dir() {
                let size_bytes = calculate_directory_size(&path).await?;
                Ok(size_bytes as f64 / (1024.0 * 1024.0)) // Convert to MB
            } else {
                Ok(0.0)
            }
        }
        RepositoryType::Git | RepositoryType::GitLab | RepositoryType::GitHub => {
            // For Git repos, estimate based on .git directory if cloned locally
            // Or use a placeholder calculation
            // In practice, you might want to track this during crawling
            Ok(0.0) // Placeholder - would need actual implementation
        }
    }
}

// Helper function to recursively calculate directory size
async fn calculate_directory_size(dir: &PathBuf) -> Result<u64> {
    use tokio::fs;

    let mut total_size = 0u64;
    let mut read_dir = fs::read_dir(dir).await?;

    while let Some(entry) = read_dir.next_entry().await? {
        let metadata = entry.metadata().await?;

        if metadata.is_file() {
            total_size += metadata.len();
        } else if metadata.is_dir() {
            // Skip hidden directories and common large directories
            let dir_name = entry.file_name();
            let dir_name_str = dir_name.to_string_lossy();

            if !dir_name_str.starts_with('.')
                && !matches!(dir_name_str.as_ref(), "node_modules" | "target" | "build" | "dist")
            {
                if let Ok(subdir_size) = Box::pin(calculate_directory_size(&entry.path())).await {
                    total_size += subdir_size;
                }
            }
        }
    }

    Ok(total_size)
}

// Helper function to get file count for a repository
async fn get_repository_file_count(repository: &Repository, _app_state: &AppState) -> Result<i64> {
    // Try to get count from search index if available
    // For now, return estimated count based on repository type
    match repository.repository_type {
        RepositoryType::FileSystem => {
            let path = PathBuf::from(&repository.url);
            if path.exists() && path.is_dir() {
                count_files_in_directory(&path).await
            } else {
                Ok(0)
            }
        }
        RepositoryType::Git | RepositoryType::GitLab | RepositoryType::GitHub => {
            // Could query the search index for files from this project
            Ok(0) // Placeholder
        }
    }
}

// Helper function to count files in directory
async fn count_files_in_directory(dir: &PathBuf) -> Result<i64> {
    use tokio::fs;

    let mut file_count = 0i64;
    let mut read_dir = fs::read_dir(dir).await?;

    while let Some(entry) = read_dir.next_entry().await? {
        let metadata = entry.metadata().await?;

        if metadata.is_file() {
            // Check if it's a supported file type
            if let Some(extension) = entry.path().extension() {
                if is_supported_extension(extension.to_string_lossy().as_ref()) {
                    file_count += 1;
                }
            }
        } else if metadata.is_dir() {
            let dir_name = entry.file_name();
            let dir_name_str = dir_name.to_string_lossy();

            if !dir_name_str.starts_with('.')
                && !matches!(dir_name_str.as_ref(), "node_modules" | "target" | "build" | "dist")
            {
                if let Ok(subdir_count) = Box::pin(count_files_in_directory(&entry.path())).await {
                    file_count += subdir_count;
                }
            }
        }
    }

    Ok(file_count)
}

// Helper function to check if file extension is supported
fn is_supported_extension(extension: &str) -> bool {
    const SUPPORTED_EXTENSIONS: &[&str] = &[
        "rs",
        "py",
        "js",
        "ts",
        "java",
        "c",
        "cpp",
        "h",
        "hpp",
        "go",
        "rb",
        "php",
        "cs",
        "swift",
        "kt",
        "scala",
        "clj",
        "hs",
        "ml",
        "fs",
        "elm",
        "dart",
        "vue",
        "jsx",
        "tsx",
        "html",
        "css",
        "scss",
        "less",
        "sql",
        "sh",
        "bash",
        "zsh",
        "fish",
        "ps1",
        "bat",
        "cmd",
        "dockerfile",
        "yaml",
        "yml",
        "json",
        "toml",
        "xml",
        "md",
        "txt",
        "cfg",
        "conf",
        "ini",
        "properties",
        "gradle",
        "maven",
        "pom",
        "sbt",
        "cmake",
        "makefile",
    ];

    SUPPORTED_EXTENSIONS.contains(&extension.to_lowercase().as_str())
}

// Rest of the repository API functions would be implemented here...
// (create_repository, update_repository, delete_repository, etc.)
// For brevity, I'm focusing on the disk size functionality

async fn get_repositories_stats(
    _user: AdminUser,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let repo_repository = RepositoryRepository::new(app_state.database.pool().clone());

    match repo_repository.list_repositories().await {
        Ok(repositories) => {
            let mut total_disk_size_mb = 0.0;
            let mut total_files = 0i64;

            for repo in &repositories {
                if let Ok(disk_size) = calculate_repository_disk_size(repo).await {
                    total_disk_size_mb += disk_size;
                }
                if let Ok(file_count) = get_repository_file_count(repo, &app_state).await {
                    total_files += file_count;
                }
            }

            let stats = serde_json::json!({
                "total_repositories": repositories.len(),
                "total_disk_size_mb": total_disk_size_mb,
                "total_files": total_files,
                "average_size_per_repo_mb": if repositories.is_empty() { 0.0 } else { total_disk_size_mb / repositories.len() as f64 }
            });

            Ok(Json(stats))
        }
        Err(e) => {
            error!("Failed to get repository stats: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_repository(
    _user: AdminUser,
    State(app_state): State<AppState>,
    body: Bytes,
) -> Result<Json<Repository>, StatusCode> {
    debug!("Received raw JSON body: {:?}", String::from_utf8_lossy(&body));

    let request: CreateRepositoryRequest = match serde_json::from_slice(&body) {
        Ok(req) => {
            debug!("Successfully parsed create repository request: {:?}", req);
            req
        }
        Err(e) => {
            error!("Failed to parse JSON request: {}", e);
            error!("Raw body: {}", String::from_utf8_lossy(&body));
            return Err(StatusCode::UNPROCESSABLE_ENTITY);
        }
    };
    let repo_repository = RepositoryRepository::new(app_state.database.pool().clone());

    // Validate GitHub namespace if provided
    if let Some(ref github_namespace) = request.github_namespace {
        if let Err(e) = validate_github_namespace(github_namespace) {
            error!("Invalid GitHub namespace '{}': {}", github_namespace, e);
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    // Encrypt access token if provided
    let encrypted_token = if let Some(token) = &request.access_token {
        match app_state.encryption_service.encrypt(token) {
            Ok(encrypted) => Some(encrypted),
            Err(e) => {
                error!("Failed to encrypt access token: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        }
    } else {
        None
    };

    let repository = Repository {
        id: Uuid::new_v4(),
        name: request.name,
        url: request.url,
        repository_type: request.repository_type,
        branch: request.branch,
        enabled: request.enabled.unwrap_or(true),
        access_token: encrypted_token,
        gitlab_namespace: request.gitlab_namespace,
        is_group: request.is_group.unwrap_or(false),
        last_crawled: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        auto_crawl_enabled: request.auto_crawl_enabled.unwrap_or(false),
        cron_schedule: request.cron_schedule,
        next_crawl_at: None,
        crawl_frequency_hours: request.crawl_frequency_hours,
        max_crawl_duration_minutes: request.max_crawl_duration_minutes,
        last_crawl_duration_seconds: None,
        gitlab_excluded_projects: request.gitlab_excluded_projects,
        gitlab_excluded_patterns: request.gitlab_excluded_patterns,
        github_namespace: request.github_namespace,
        github_excluded_repositories: request.github_excluded_repositories,
        github_excluded_patterns: request.github_excluded_patterns,
        crawl_state: Some("idle".to_string()),
        last_processed_project: None,
        crawl_started_at: None,
    };

    match repo_repository.create_repository(&repository).await {
        Ok(created_repo) => {
            info!("Created repository: {} ({})", created_repo.name, created_repo.id);
            Ok(Json(created_repo))
        }
        Err(e) => {
            error!("Failed to create repository: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_repository(
    _user: AdminUser,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Repository>, StatusCode> {
    let repo_repository = RepositoryRepository::new(app_state.database.pool().clone());

    match repo_repository.get_repository(id).await {
        Ok(Some(repository)) => Ok(Json(repository)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get repository {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_repository(
    _user: AdminUser,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateRepositoryRequest>,
) -> Result<Json<Repository>, StatusCode> {
    info!(
        "Update repository request received - ID: {}, Request: {:?}",
        id, request
    );
    let repo_repository = RepositoryRepository::new(app_state.database.pool().clone());

    // Get existing repository
    let mut repository = match repo_repository.get_repository(id).await {
        Ok(Some(repo)) => repo,
        Ok(None) => return Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get repository {}: {}", id, e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Track if name is being changed for search index update
    let old_name = repository.name.clone();
    let mut name_changed = false;

    // Update fields if provided
    if let Some(name) = request.name {
        if name != old_name {
            name_changed = true;
        }
        repository.name = name;
    }
    if let Some(url) = request.url {
        repository.url = url;
    }
    if let Some(repository_type) = request.repository_type {
        repository.repository_type = repository_type;
    }
    if let Some(branch) = request.branch {
        repository.branch = Some(branch);
    }
    if let Some(enabled) = request.enabled {
        repository.enabled = enabled;
    }
    if let Some(gitlab_namespace) = request.gitlab_namespace {
        repository.gitlab_namespace = Some(gitlab_namespace);
    }
    if let Some(is_group) = request.is_group {
        repository.is_group = is_group;
    }
    // Track if scheduling was changed
    let scheduling_changed = request.auto_crawl_enabled.is_some()
        || request.cron_schedule.is_some()
        || request.crawl_frequency_hours.is_some();

    if let Some(auto_crawl_enabled) = request.auto_crawl_enabled {
        repository.auto_crawl_enabled = auto_crawl_enabled;
    }

    // Handle scheduling: cron_schedule and crawl_frequency_hours are mutually exclusive
    match (&request.cron_schedule, &request.crawl_frequency_hours) {
        (Some(cron), _) => {
            // Cron mode: set cron_schedule, clear crawl_frequency_hours
            repository.cron_schedule = Some(cron.clone());
            repository.crawl_frequency_hours = None;
        }
        (None, Some(freq)) => {
            // Frequency mode: set crawl_frequency_hours, clear cron_schedule
            repository.crawl_frequency_hours = Some(*freq);
            repository.cron_schedule = None;
        }
        (None, None) => {
            // Neither provided: don't change
        }
    }
    if let Some(max_crawl_duration_minutes) = request.max_crawl_duration_minutes {
        repository.max_crawl_duration_minutes = Some(max_crawl_duration_minutes);
    }
    if let Some(gitlab_excluded_projects) = request.gitlab_excluded_projects {
        repository.gitlab_excluded_projects = Some(gitlab_excluded_projects);
    }
    if let Some(gitlab_excluded_patterns) = request.gitlab_excluded_patterns {
        repository.gitlab_excluded_patterns = Some(gitlab_excluded_patterns);
    }
    if let Some(github_namespace) = request.github_namespace {
        // Validate GitHub namespace before updating
        if let Err(e) = validate_github_namespace(&github_namespace) {
            error!("Invalid GitHub namespace '{}': {}", github_namespace, e);
            return Err(StatusCode::BAD_REQUEST);
        }
        repository.github_namespace = Some(github_namespace);
    }
    if let Some(github_excluded_repositories) = request.github_excluded_repositories {
        repository.github_excluded_repositories = Some(github_excluded_repositories);
    }
    if let Some(github_excluded_patterns) = request.github_excluded_patterns {
        repository.github_excluded_patterns = Some(github_excluded_patterns);
    }

    // Handle access token update with encryption
    if let Some(access_token) = request.access_token {
        // Check if the token is already encrypted (i.e., it's the same as what's already stored)
        if repository.access_token.as_ref() == Some(&access_token) {
            // Token is already encrypted and unchanged, don't re-encrypt
            info!("Access token unchanged, keeping existing encrypted token");
        } else {
            // This is a new token that needs to be encrypted
            repository.access_token = match app_state.encryption_service.encrypt(&access_token) {
                Ok(encrypted) => Some(encrypted),
                Err(e) => {
                    error!("Failed to encrypt access token: {}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            };
            info!("Access token updated and encrypted");
        }
    }

    repository.updated_at = Utc::now();

    match repo_repository.update_repository(id, &repository).await {
        Ok(updated_repo) => {
            // If repository name was changed, update search index
            if name_changed {
                match app_state.search_service.update_project_name(&old_name, &updated_repo.name).await {
                    Ok(updated_count) => {
                        info!(
                            "Updated {} documents in search index for repository name change: {} -> {} ({})",
                            updated_count, old_name, updated_repo.name, updated_repo.id
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Failed to update search index for repository name change {} -> {}: {}. Search results may be inconsistent until re-crawling.", 
                            old_name, updated_repo.name, e
                        );
                    }
                }
            }

            // If scheduling was changed, reschedule the repository
            if scheduling_changed {
                if let Some(scheduler) = &app_state.scheduler_service {
                    // Unschedule first (in case it was already scheduled)
                    let _ = scheduler.unschedule_repository(id).await;

                    // Reschedule if auto_crawl is enabled
                    if updated_repo.auto_crawl_enabled {
                        if let Err(e) = scheduler.schedule_repository(&updated_repo).await {
                            warn!("Failed to reschedule repository {}: {}", id, e);
                        }
                    }
                }
            }

            info!("Updated repository: {} ({})", updated_repo.name, updated_repo.id);
            info!("Returning updated repository: {:?}", updated_repo);
            Ok(Json(updated_repo))
        }
        Err(e) => {
            error!("Failed to update repository {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_repository(
    _user: AdminUser,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let repo_repository = RepositoryRepository::new(app_state.database.pool().clone());

    // Check if repository exists
    match repo_repository.get_repository(id).await {
        Ok(Some(repository)) => {
            // First, clean up the search index for this repository
            match app_state.search_service.delete_project_documents(&repository.name).await {
                Ok(deleted_count) => {
                    info!(
                        "Deleted {} documents from search index for repository: {} ({})",
                        deleted_count, repository.name, repository.id
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to delete search index documents for repository {}: {}. Proceeding with repository deletion.", 
                        repository.name, e
                    );
                    // Continue with repository deletion even if index cleanup fails
                }
            }

            // Delete the repository from database
            match repo_repository.delete_repository(id).await {
                Ok(_) => {
                    info!("Deleted repository: {} ({})", repository.name, repository.id);
                    Ok(StatusCode::NO_CONTENT)
                }
                Err(e) => {
                    error!("Failed to delete repository {}: {}", id, e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Failed to get repository {}: {}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn crawl_repository(
    _user: AdminUser,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get repository from database
    let repo_repo = RepositoryRepository::new(app_state.database.pool().clone());
    let repository = repo_repo
        .get_repository(id)
        .await
        .map_err(|e| {
            error!("Failed to get repository {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            error!("Repository not found: {}", id);
            StatusCode::NOT_FOUND
        })?;

    // Check if repository is enabled
    if !repository.enabled {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Start crawl using the crawler service
    let crawler_service = app_state.crawler_service.clone();
    let repository_clone = repository.clone();

    // Spawn crawl task in background
    tokio::spawn(async move {
        if let Err(e) = crawler_service.crawl_repository(&repository_clone).await {
            error!("Crawl failed for repository {}: {}", repository_clone.name, e);
        }
    });

    Ok(Json(serde_json::json!({
        "message": "Crawl started successfully"
    })))
}

async fn stop_crawl_repository(
    _user: AdminUser,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Get repository from database
    let repo_repo = RepositoryRepository::new(app_state.database.pool().clone());
    let repository = repo_repo
        .get_repository(id)
        .await
        .map_err(|e| {
            error!("Failed to get repository {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or_else(|| {
            error!("Repository not found: {}", id);
            StatusCode::NOT_FOUND
        })?;

    // Check if repository is currently being crawled
    let crawler_service = app_state.crawler_service.clone();
    let is_crawling = crawler_service.is_crawling(repository.id).await;

    if !is_crawling {
        return Ok(Json(serde_json::json!({
            "message": "No active crawl found for this repository",
            "repository_id": repository.id,
            "repository_name": repository.name
        })));
    }

    // Cancel the crawl
    match crawler_service.cancel_crawl(repository.id).await {
        Ok(true) => {
            info!("Crawl stopped for repository: {} ({})", repository.name, repository.id);
            Ok(Json(serde_json::json!({
                "message": "Crawl stopped successfully",
                "repository_id": repository.id,
                "repository_name": repository.name
            })))
        }
        Ok(false) => {
            warn!(
                "Failed to stop crawl for repository: {} ({})",
                repository.name, repository.id
            );
            Err(StatusCode::NOT_FOUND)
        }
        Err(e) => {
            error!("Error stopping crawl for repository {}: {}", repository.id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn test_repository_connection(
    _user: AdminUser,
    State(_app_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Implementation would go here
    Err(StatusCode::NOT_IMPLEMENTED)
}

async fn import_gitlab_projects(
    _user: AdminUser,
    State(_app_state): State<AppState>,
    Json(_request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Implementation would go here
    Err(StatusCode::NOT_IMPLEMENTED)
}

// GitHub discovery endpoint
async fn discover_github_repositories(
    _user: AdminUser,
    State(_app_state): State<AppState>,
    Json(request): Json<DiscoverGitHubRequest>,
) -> Result<Json<DiscoverGitHubResponse>, StatusCode> {
    info!(
        "Discovering GitHub repositories with namespace: {:?}",
        request.namespace
    );

    // Validate GitHub namespace if provided
    if let Some(ref namespace) = request.namespace {
        if let Err(e) = validate_github_namespace(namespace) {
            error!("Invalid GitHub namespace '{}': {}", namespace, e);
            return Err(StatusCode::BAD_REQUEST);
        }
    }

    let github_service = GitHubService::new();

    match github_service.discover_repositories(&request.access_token, request.namespace.as_deref()).await {
        Ok(repositories) => {
            info!("Successfully discovered {} GitHub repositories", repositories.len());
            Ok(Json(DiscoverGitHubResponse { repositories }))
        }
        Err(e) => {
            error!("Failed to discover GitHub repositories: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// GitHub token test endpoint
async fn test_github_token(
    _user: AdminUser,
    State(_app_state): State<AppState>,
    Json(request): Json<TestGitHubTokenRequest>,
) -> Result<Json<TestTokenResponse>, StatusCode> {
    info!("Testing GitHub token");

    let github_service = GitHubService::new();

    match github_service.test_token(&request.access_token).await {
        Ok(valid) => {
            let message = if valid {
                "GitHub token is valid".to_string()
            } else {
                "GitHub token is invalid".to_string()
            };
            info!("{}", message);
            Ok(Json(TestTokenResponse { valid, message }))
        }
        Err(e) => {
            error!("Failed to test GitHub token: {}", e);
            Ok(Json(TestTokenResponse {
                valid: false,
                message: format!("Error testing token: {}", e),
            }))
        }
    }
}

// GitLab discovery endpoint
async fn discover_gitlab_repositories(
    _user: AdminUser,
    State(_app_state): State<AppState>,
    Json(request): Json<DiscoverGitLabRequest>,
) -> Result<Json<DiscoverGitLabResponse>, StatusCode> {
    info!(
        "Discovering GitLab projects from {} with namespace: {:?}",
        request.gitlab_url, request.namespace
    );

    let gitlab_service = GitLabService::new();

    match gitlab_service
        .discover_projects(&request.gitlab_url, &request.access_token, request.namespace.as_deref())
        .await
    {
        Ok(projects) => {
            info!("Successfully discovered {} GitLab projects", projects.len());
            Ok(Json(DiscoverGitLabResponse { projects }))
        }
        Err(e) => {
            error!("Failed to discover GitLab projects: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// GitLab token test endpoint
async fn test_gitlab_token(
    _user: AdminUser,
    State(_app_state): State<AppState>,
    Json(request): Json<TestGitLabTokenRequest>,
) -> Result<Json<TestTokenResponse>, StatusCode> {
    info!("Testing GitLab token for URL: {}", request.gitlab_url);

    let gitlab_service = GitLabService::new();

    match gitlab_service.test_token(&request.gitlab_url, &request.access_token).await {
        Ok(valid) => {
            let message = if valid {
                "GitLab token is valid".to_string()
            } else {
                "GitLab token is invalid".to_string()
            };
            info!("{}", message);
            Ok(Json(TestTokenResponse { valid, message }))
        }
        Err(e) => {
            error!("Failed to test GitLab token: {}", e);
            Ok(Json(TestTokenResponse {
                valid: false,
                message: format!("Error testing token: {}", e),
            }))
        }
    }
}

async fn bulk_enable_repositories(
    _user: AdminUser,
    State(_app_state): State<AppState>,
    Json(_request): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    // Implementation would go here
    Err(StatusCode::NOT_IMPLEMENTED)
}

async fn bulk_disable_repositories(
    _user: AdminUser,
    State(_app_state): State<AppState>,
    Json(_request): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    // Implementation would go here
    Err(StatusCode::NOT_IMPLEMENTED)
}

async fn bulk_crawl_repositories(
    _user: AdminUser,
    State(_app_state): State<AppState>,
    Json(_request): Json<serde_json::Value>,
) -> Result<StatusCode, StatusCode> {
    // Implementation would go here
    Err(StatusCode::NOT_IMPLEMENTED)
}

async fn bulk_delete_repositories(
    _user: AdminUser,
    State(app_state): State<AppState>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let repo_repository = RepositoryRepository::new(app_state.database.pool().clone());

    // Extract repository IDs from request
    let repository_ids: Vec<String> = match request.get("repository_ids") {
        Some(ids) => match ids.as_array() {
            Some(arr) => arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect(),
            None => return Err(StatusCode::BAD_REQUEST),
        },
        None => return Err(StatusCode::BAD_REQUEST),
    };

    if repository_ids.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut successful_deletions = 0;
    let mut failed_deletions = 0;
    let mut index_cleanup_failures = 0;

    for repo_id_str in repository_ids {
        let repo_id = match Uuid::parse_str(&repo_id_str) {
            Ok(id) => id,
            Err(_) => {
                failed_deletions += 1;
                continue;
            }
        };

        // Get repository info before deletion for index cleanup
        match repo_repository.get_repository(repo_id).await {
            Ok(Some(repository)) => {
                // Clean up search index first
                match app_state.search_service.delete_project_documents(&repository.name).await {
                    Ok(deleted_count) => {
                        debug!(
                            "Deleted {} documents from search index for repository: {} ({})",
                            deleted_count, repository.name, repository.id
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Failed to delete search index documents for repository {}: {}. Proceeding with repository deletion.", 
                            repository.name, e
                        );
                        index_cleanup_failures += 1;
                    }
                }

                // Delete repository from database
                match repo_repository.delete_repository(repo_id).await {
                    Ok(_) => {
                        info!("Deleted repository: {} ({})", repository.name, repository.id);
                        successful_deletions += 1;
                    }
                    Err(e) => {
                        error!("Failed to delete repository {}: {}", repo_id, e);
                        failed_deletions += 1;
                    }
                }
            }
            Ok(None) => {
                // Repository not found
                failed_deletions += 1;
            }
            Err(e) => {
                error!("Failed to get repository {}: {}", repo_id, e);
                failed_deletions += 1;
            }
        }
    }

    let response = serde_json::json!({
        "successful": successful_deletions,
        "failed": failed_deletions,
        "index_cleanup_failures": index_cleanup_failures
    });

    Ok(Json(response))
}

async fn get_active_crawl_progress(
    _user: AdminUser,
    State(app_state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let active_progress = app_state.progress_tracker.get_all_active_progress().await;

    let progress_data: Vec<serde_json::Value> = active_progress
        .iter()
        .map(|progress| {
            serde_json::json!({
                "repository_id": progress.repository_id,
                "repository_name": progress.repository_name,
                "status": progress.status,
                "files_processed": progress.files_processed,
                "files_total": progress.files_total,
                "files_indexed": progress.files_indexed,
                "current_file": progress.current_file,
                "percentage": progress.progress_percentage,
                "error_message": progress.error_message,
                "started_at": progress.started_at,
                "updated_at": progress.updated_at,
                "completed_at": progress.completed_at,
                "projects_processed": progress.projects_processed,
                "projects_total": progress.projects_total,
                "current_project": progress.current_project,
                "current_project_files_processed": progress.current_project_files_processed,
                "current_project_files_total": progress.current_project_files_total
            })
        })
        .collect();

    Ok(Json(serde_json::json!({
        "active_progress": progress_data
    })))
}
