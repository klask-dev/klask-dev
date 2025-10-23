use crate::auth::extractors::{AdminUser, AppState};
use crate::repositories::{user_repository::UserStats, UserRepository};
use crate::services::seeding::{SeedingService, SeedingStats};
use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use tracing::{debug, error, info};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStats {
    pub uptime_seconds: u64,
    pub version: String,
    pub environment: String,
    pub database_status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryStats {
    pub total_repositories: i64,
    pub enabled_repositories: i64,
    pub disabled_repositories: i64,
    pub git_repositories: i64,
    pub gitlab_repositories: i64,
    pub filesystem_repositories: i64,
    pub recently_crawled: i64, // Last 24h
    pub never_crawled: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchStats {
    pub total_documents: i64,
    pub index_size_mb: f64,
    pub avg_search_time_ms: Option<f64>,
    pub popular_queries: Vec<QueryStat>,
    pub documents_by_repository: Vec<RepositoryDocumentCount>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryStat {
    pub query: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositoryDocumentCount {
    pub repository_name: String,
    pub document_count: i64,
    pub repository_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentStats {
    pub total_files: i64,
    pub total_size_bytes: i64,
    pub files_by_extension: Vec<ExtensionStat>,
    pub files_by_project: Vec<ProjectStat>,
    pub recent_additions: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtensionStat {
    pub extension: String,
    pub count: i64,
    pub total_size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectStat {
    pub project: String,
    pub file_count: i64,
    pub total_size: i64,
    pub disk_size_mb: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentActivity {
    pub recent_users: Vec<RecentUser>,
    pub recent_repositories: Vec<RecentRepository>,
    pub recent_crawls: Vec<RecentCrawl>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentUser {
    pub username: String,
    pub email: String,
    pub last_seen: DateTime<Utc>,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentRepository {
    pub name: String,
    pub url: String,
    pub repository_type: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentCrawl {
    pub repository_name: String,
    pub last_crawled: Option<DateTime<Utc>>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AdminDashboardData {
    pub system: SystemStats,
    pub users: UserStats,
    pub repositories: RepositoryStats,
    pub content: ContentStats,
    pub search: SearchStats,
    pub recent_activity: RecentActivity,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IndexResetResponse {
    pub success: bool,
    pub message: String,
    pub documents_before: u64,
    pub documents_after: u64,
}

pub async fn create_router() -> Result<Router<AppState>> {
    let router = Router::new()
        .route("/dashboard", get(get_dashboard_data))
        .route("/system", get(get_system_stats))
        .route("/users/stats", get(get_user_stats))
        .route("/repositories/stats", get(get_repository_stats))
        .route("/content/stats", get(get_content_stats))
        .route("/search/stats", get(get_search_stats))
        .route("/activity/recent", get(get_recent_activity))
        .route("/seed", post(seed_database))
        .route("/seed/clear", post(clear_seed_data))
        .route("/seed/stats", get(get_seed_stats))
        .route("/search/reset-index", post(reset_search_index));

    Ok(router)
}

async fn get_dashboard_data(State(app_state): State<AppState>) -> Result<Json<AdminDashboardData>, StatusCode> {
    debug!("Getting dashboard data for admin user");
    let pool = app_state.database.pool().clone();

    // Gather all stats in parallel using tokio::join!
    let (system_result, users_result, repos_result, content_result, search_result, activity_result) = tokio::join!(
        get_system_stats_impl(&app_state),
        get_user_stats_impl(&pool),
        get_repository_stats_impl(&pool),
        get_content_stats_impl(&pool),
        get_search_stats_impl(&app_state),
        get_recent_activity_impl(&pool)
    );

    let system = system_result.map_err(|e| {
        error!("Failed to get system stats: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let users = users_result.map_err(|e| {
        error!("Failed to get user stats: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let repositories = repos_result.map_err(|e| {
        error!("Failed to get repository stats: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let content = content_result.map_err(|e| {
        error!("Failed to get content stats: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let search = search_result.map_err(|e| {
        error!("Failed to get search stats: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let recent_activity = activity_result.map_err(|e| {
        error!("Failed to get recent activity: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let dashboard_data = AdminDashboardData { system, users, repositories, content, search, recent_activity };

    info!("Successfully generated dashboard data");
    Ok(Json(dashboard_data))
}

async fn get_system_stats(State(app_state): State<AppState>) -> Result<Json<SystemStats>, StatusCode> {
    match get_system_stats_impl(&app_state).await {
        Ok(stats) => Ok(Json(stats)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_user_stats(State(app_state): State<AppState>) -> Result<Json<UserStats>, StatusCode> {
    let pool = app_state.database.pool().clone();
    match get_user_stats_impl(&pool).await {
        Ok(stats) => Ok(Json(stats)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_repository_stats(State(app_state): State<AppState>) -> Result<Json<RepositoryStats>, StatusCode> {
    let pool = app_state.database.pool().clone();
    match get_repository_stats_impl(&pool).await {
        Ok(stats) => Ok(Json(stats)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_search_stats(State(app_state): State<AppState>) -> Result<Json<SearchStats>, StatusCode> {
    match get_search_stats_impl(&app_state).await {
        Ok(stats) => Ok(Json(stats)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_content_stats(State(app_state): State<AppState>) -> Result<Json<ContentStats>, StatusCode> {
    let pool = app_state.database.pool().clone();
    match get_content_stats_impl(&pool).await {
        Ok(stats) => Ok(Json(stats)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_recent_activity(State(app_state): State<AppState>) -> Result<Json<RecentActivity>, StatusCode> {
    let pool = app_state.database.pool().clone();
    match get_recent_activity_impl(&pool).await {
        Ok(activity) => Ok(Json(activity)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// Implementation functions

async fn get_system_stats_impl(app_state: &AppState) -> Result<SystemStats> {
    let database_status = match app_state.database.health_check().await {
        Ok(_) => "Connected".to_string(),
        Err(_) => "Disconnected".to_string(),
    };

    // Calculate uptime in seconds since server startup
    let uptime_seconds = app_state.startup_time.elapsed().as_secs();

    Ok(SystemStats {
        uptime_seconds,
        version: env!("CARGO_PKG_VERSION").to_string(),
        environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
        database_status,
    })
}

async fn get_user_stats_impl(pool: &PgPool) -> Result<UserStats> {
    let user_repository = UserRepository::new(pool.clone());
    user_repository.get_user_stats().await
}

async fn get_repository_stats_impl(pool: &PgPool) -> Result<RepositoryStats> {
    let total_repositories = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM repositories").fetch_one(pool).await?;

    let enabled_repositories =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM repositories WHERE enabled = true").fetch_one(pool).await?;

    let disabled_repositories = total_repositories - enabled_repositories;

    let git_repositories =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM repositories WHERE repository_type = 'Git'")
            .fetch_one(pool)
            .await?;

    let gitlab_repositories =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM repositories WHERE repository_type = 'GitLab'")
            .fetch_one(pool)
            .await?;

    let filesystem_repositories =
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM repositories WHERE repository_type = 'FileSystem'")
            .fetch_one(pool)
            .await?;

    let recently_crawled = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM repositories WHERE last_crawled > CURRENT_TIMESTAMP - INTERVAL '24 hours'",
    )
    .fetch_one(pool)
    .await?;

    let never_crawled = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM repositories WHERE last_crawled IS NULL")
        .fetch_one(pool)
        .await?;

    Ok(RepositoryStats {
        total_repositories,
        enabled_repositories,
        disabled_repositories,
        git_repositories,
        gitlab_repositories,
        filesystem_repositories,
        recently_crawled,
        never_crawled,
    })
}

async fn get_search_stats_impl(app_state: &AppState) -> Result<SearchStats> {
    // Get document count from search service
    let total_documents = match app_state.search_service.get_document_count() {
        Ok(count) => count as i64,
        Err(_) => 0,
    };

    // Get actual index size in MB
    let index_size_mb = app_state.search_service.get_index_size_mb();

    // Get documents by repository from advanced metrics
    let documents_by_repository = match app_state.search_service.get_advanced_metrics() {
        Ok(metrics) => {
            let pool = app_state.database.pool().clone();
            let repo_names: Vec<String> = metrics.documents_by_repository.keys().cloned().collect();

            // Single query to get all repository types (avoid N+1 problem)
            let repo_types: std::collections::HashMap<String, String> = if !repo_names.is_empty() {
                sqlx::query_as::<_, (String, String)>(
                    "SELECT name, repository_type::TEXT FROM repositories WHERE name = ANY($1)",
                )
                .bind(&repo_names)
                .fetch_all(&pool)
                .await
                .unwrap_or_default()
                .into_iter()
                .collect()
            } else {
                std::collections::HashMap::new()
            };

            // Build the result
            metrics
                .documents_by_repository
                .into_iter()
                .map(|(repo_name, doc_count)| RepositoryDocumentCount {
                    repository_name: repo_name.clone(),
                    document_count: doc_count as i64,
                    repository_type: repo_types.get(&repo_name).cloned(),
                })
                .collect()
        }
        Err(_) => vec![],
    };

    // TODO: Implement actual search metrics tracking
    // For now, return basic stats with real index size
    Ok(SearchStats {
        total_documents,
        index_size_mb,
        avg_search_time_ms: None, // TODO: Track search performance
        popular_queries: vec![],  // TODO: Track popular queries
        documents_by_repository,
    })
}

async fn get_content_stats_impl(_pool: &PgPool) -> Result<ContentStats> {
    // Files are now stored only in Tantivy search index, not in database
    // Return default/placeholder values since this data is not available
    // TODO: Extract file statistics from Tantivy index if needed

    Ok(ContentStats {
        total_files: 0,
        total_size_bytes: 0,
        files_by_extension: vec![],
        files_by_project: vec![],
        recent_additions: 0,
    })
}

async fn get_recent_activity_impl(pool: &PgPool) -> Result<RecentActivity> {
    // Recent users based on activity (last login or last activity)
    let recent_users_rows = sqlx::query(
        "SELECT username, email,
                COALESCE(last_activity, last_login, created_at) as last_seen,
                role::TEXT as role
         FROM users
         WHERE last_login IS NOT NULL OR last_activity IS NOT NULL
         ORDER BY COALESCE(last_activity, last_login, created_at) DESC
         LIMIT 5",
    )
    .fetch_all(pool)
    .await?;

    let recent_users = recent_users_rows
        .into_iter()
        .map(|row| RecentUser {
            username: row.get("username"),
            email: row.get("email"),
            last_seen: row.get("last_seen"),
            role: row.get::<Option<String>, _>("role").unwrap_or_else(|| "User".to_string()),
        })
        .collect();

    // Recent repositories (last 7 days)
    let recent_repositories_rows = sqlx::query(
        "SELECT name, url, repository_type::TEXT as repository_type, created_at
         FROM repositories
         WHERE created_at > CURRENT_TIMESTAMP - INTERVAL '7 days'
         ORDER BY created_at DESC
         LIMIT 5",
    )
    .fetch_all(pool)
    .await?;

    let recent_repositories = recent_repositories_rows
        .into_iter()
        .map(|row| RecentRepository {
            name: row.get("name"),
            url: row.get("url"),
            repository_type: row.get::<Option<String>, _>("repository_type").unwrap_or_else(|| "Unknown".to_string()),
            created_at: row.get("created_at"),
        })
        .collect();

    // Recent crawls (last crawled repositories)
    let recent_crawls_rows = sqlx::query(
        "SELECT name, last_crawled 
         FROM repositories 
         WHERE last_crawled IS NOT NULL 
         ORDER BY last_crawled DESC 
         LIMIT 10",
    )
    .fetch_all(pool)
    .await?;

    let recent_crawls = recent_crawls_rows
        .into_iter()
        .map(|row| RecentCrawl {
            repository_name: row.get("name"),
            last_crawled: row.get("last_crawled"),
            status: "Completed".to_string(), // TODO: Track actual crawl status
        })
        .collect();

    Ok(RecentActivity { recent_users, recent_repositories, recent_crawls })
}

// Seeding endpoints

async fn seed_database(State(app_state): State<AppState>) -> Result<Json<SeedingStats>, StatusCode> {
    info!("Admin user requested database seeding");
    let pool = app_state.database.pool().clone();
    let seeding_service = SeedingService::new(pool);

    seeding_service.seed_all().await.map_err(|e| {
        error!("Database seeding failed: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Database seeding completed successfully");
    let stats = seeding_service.get_stats().await.map_err(|e| {
        error!("Failed to get seeding stats: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(stats))
}

async fn clear_seed_data(State(app_state): State<AppState>) -> Result<Json<SeedingStats>, StatusCode> {
    info!("Admin user requested seed data clearing");
    let pool = app_state.database.pool().clone();
    let seeding_service = SeedingService::new(pool);

    seeding_service.clear_all().await.map_err(|e| {
        error!("Failed to clear seed data: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    info!("Seed data cleared successfully");
    let stats = seeding_service.get_stats().await.map_err(|e| {
        error!("Failed to get seeding stats after clearing: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(stats))
}

async fn get_seed_stats(State(app_state): State<AppState>) -> Result<Json<SeedingStats>, StatusCode> {
    debug!("Getting seeding stats for admin user");
    let pool = app_state.database.pool().clone();
    let seeding_service = SeedingService::new(pool);

    let stats = seeding_service.get_stats().await.map_err(|e| {
        error!("Failed to get seeding stats: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(stats))
}
// Search index management endpoints

async fn reset_search_index(
    _admin_user: AdminUser,
    State(app_state): State<AppState>,
) -> Result<Json<IndexResetResponse>, StatusCode> {
    info!("Admin user requested search index reset");

    // Get document count before reset
    let documents_before = app_state.search_service.get_document_count().unwrap_or(0);

    // Reset the index
    match app_state.search_service.reset_index().await {
        Ok(_) => {
            info!("Search index reset successfully");

            // Get document count after reset (should be 0)
            let documents_after = app_state.search_service.get_document_count().unwrap_or(0);

            Ok(Json(IndexResetResponse {
                success: true,
                message: "Search index has been reset successfully".to_string(),
                documents_before,
                documents_after,
            }))
        }
        Err(e) => {
            error!("Failed to reset search index: {:?}", e);
            Ok(Json(IndexResetResponse {
                success: false,
                message: format!("Failed to reset index: {}", e),
                documents_before,
                documents_after: documents_before,
            }))
        }
    }
}
