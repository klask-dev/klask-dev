// Test program to verify search functionality works with commit fix
use anyhow::Result;
use klask_rs::models::*;
use klask_rs::services::crawler::CrawlerService;
use klask_rs::services::encryption::EncryptionService;
use klask_rs::services::progress::ProgressTracker;
use klask_rs::services::search::{SearchQuery, SearchService};
use sqlx::PgPool;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // Connect to database
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://klask:klask@localhost:5432/klask_rs".to_string());

    info!("Connecting to database: {}", database_url);
    let pool = PgPool::connect(&database_url).await?;

    // Initialize search service
    let search_service = Arc::new(SearchService::new("./test_search_index")?);

    // Clear any existing index
    search_service.clear_index().await?;
    info!("Cleared existing search index");

    // Initialize progress tracker
    let progress_tracker = Arc::new(ProgressTracker::new());

    // Create a test encryption service
    let encryption_service = Arc::new(EncryptionService::new("test-encryption-key-32bytes").unwrap());

    // Initialize crawler service
    let crawler_service = CrawlerService::new(
        pool.clone(),
        search_service.clone(),
        progress_tracker.clone(),
        encryption_service,
        std::env::temp_dir().join("klask-crawler").to_string_lossy().to_string(),
    )?;

    // Create a test repository entry for /home/jeremie/temp/
    let test_repo = Repository {
        id: Uuid::new_v4(),
        name: "test-temp".to_string(),
        url: "/home/jeremie/temp/".to_string(),
        repository_type: RepositoryType::FileSystem,
        branch: None,
        enabled: true,
        access_token: None,
        gitlab_namespace: None,
        is_group: false,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_crawled: None,
        // Scheduling fields
        auto_crawl_enabled: false,
        cron_schedule: None,
        next_crawl_at: None,
        crawl_frequency_hours: None,
        max_crawl_duration_minutes: Some(60),
        last_crawl_duration_seconds: None,
        gitlab_excluded_projects: None,
        gitlab_excluded_patterns: None,
        github_namespace: None,
        github_excluded_repositories: None,
        github_excluded_patterns: None,
        crawl_state: None,
        last_processed_project: None,
        crawl_started_at: None,
    };

    info!("Starting crawler for test repository: {}", test_repo.url);

    // Crawl the repository - this should now include the commit fix
    match crawler_service.crawl_repository(&test_repo).await {
        Ok(()) => {
            info!("✅ Repository crawl completed successfully!");
        }
        Err(e) => {
            error!("❌ Repository crawl failed: {}", e);
            return Err(e);
        }
    }

    // Now test search functionality
    info!("Testing search for 'server'...");

    let search_query = SearchQuery {
        query: "server".to_string(),
        repository_filter: None,
        project_filter: None,
        version_filter: None,
        extension_filter: None,
        limit: 10,
        offset: 0,
        include_facets: false,
    };

    match search_service.search(search_query).await {
        Ok(results) => {
            info!("✅ Search completed! Found {} results", results.results.len());

            if results.results.is_empty() {
                warn!("⚠️  No results found for 'server' - this indicates the commit fix may not be working");
                return Ok(());
            }

            // Display results
            for (i, result) in results.results.iter().enumerate() {
                info!("Result {}: {}", i + 1, result.file_path);
                info!("  Project: {}", result.project);
                info!("  Extension: {}", result.extension);
                info!("  Score: {:.2}", result.score);
                info!(
                    "  Snippet: {}...",
                    result.content_snippet.chars().take(100).collect::<String>()
                );
                info!("---");
            }

            info!("🎉 Search functionality is working correctly!");
        }
        Err(e) => {
            error!("❌ Search failed: {}", e);
            return Err(e);
        }
    }

    // Clean up test index
    let _ = std::fs::remove_dir_all("./test_search_index");
    info!("Cleaned up test index");

    Ok(())
}
