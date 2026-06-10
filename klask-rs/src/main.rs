mod api;
mod auth;
mod config;
mod database;
mod models;
mod repositories;
mod services;
mod utils;
mod version;

use anyhow::Result;
use auth::{extractors::AppState, jwt::JwtService};
use axum::{Json, Router, routing::get};
use config::AppConfig;
use database::Database;
use serde::Serialize;
use services::{
    SearchService, crawler::CrawlerService, encryption::EncryptionService, progress::ProgressTracker,
    scheduler::SchedulerService,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Serialize)]
struct VersionInfo {
    version: String,
    commit: Option<String>,
    timestamp: Option<String>,
}

/// Sanitize database URL by masking password for logging
fn sanitize_db_url(url: &str) -> String {
    if let Some(at_pos) = url.rfind('@') {
        // Find the password part (between :// and @)
        if let Some(scheme_end) = url.find("://") {
            let scheme_part = &url[..scheme_end + 3]; // "postgresql://"
            let host_part = &url[at_pos..]; // "@host:port/db"

            // Extract username (between :// and first :)
            let credentials_part = &url[scheme_end + 3..at_pos];
            if let Some(colon_pos) = credentials_part.find(':') {
                let username = &credentials_part[..colon_pos];
                return format!("{}{}:****{}", scheme_part, username, host_part);
            }
        }
    }
    // If parsing fails, just mask the entire thing for safety
    "postgresql://****:****@****".to_string()
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    // Build the filter with quiet modules first, then apply RUST_LOG or defaults
    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "klask_rs=debug,tantivy=info,sqlx=warn".to_string());
    let filter_str = format!("tantivy::directory::managed_directory=off,{}", rust_log);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(filter_str))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Capture startup time
    let startup_time = Instant::now();

    // Load configuration
    let config = AppConfig::new()?;
    let bind_address = format!("{}:{}", config.server.host, config.server.port);

    info!("Starting Klask-RS server on {}", bind_address);

    // Initialize database
    let database = match Database::new(&config.database.url, config.database.max_connections).await {
        Ok(db) => {
            info!("Database connected successfully");
            db
        }
        Err(e) => {
            error!(
                "Failed to connect to database {}: {}",
                sanitize_db_url(&config.database.url),
                e
            );
            info!("Continuing without database connection for development");
            // For development, we'll create a dummy database
            return Err(e);
        }
    };

    // Initialize search service
    let search_service = match SearchService::new(&config.search.index_dir) {
        Ok(service) => {
            info!("Search service initialized successfully at {}", config.search.index_dir);
            service
        }
        Err(e) => {
            error!("Failed to initialize search service: {}", e);
            return Err(e);
        }
    };

    // Initialize JWT service
    let jwt_service = match JwtService::new(&config.auth) {
        Ok(service) => {
            info!("JWT service initialized successfully");
            service
        }
        Err(e) => {
            error!("Failed to initialize JWT service: {}", e);
            return Err(e);
        }
    };

    // Initialize encryption service
    let encryption_service = match EncryptionService::new_from_env(database.pool()).await {
        Ok(service) => Arc::new(service),
        Err(e) => {
            error!("Failed to initialize encryption service: {}", e);
            return Err(e);
        }
    };

    // Initialize progress tracker
    let progress_tracker = Arc::new(ProgressTracker::new());
    info!("Progress tracker initialized successfully");

    // Initialize crawler service
    let search_service_arc = Arc::new(search_service);
    let crawler_service = match CrawlerService::new(
        database.pool().clone(),
        search_service_arc.clone(),
        progress_tracker.clone(),
        encryption_service.clone(),
        config.crawler.temp_dir.clone(),
        config.crawler.git_clone_timeout_secs,
        config.crawler.git_fetch_timeout_secs,
    ) {
        Ok(service) => {
            info!("Crawler service initialized successfully");

            // Check for incomplete crawls and resume them in background
            // This must not block server startup
            // Can be disabled with KLASK_SKIP_RESUME_CRAWLS=true (useful after index rebuild)
            let skip_resume =
                std::env::var("KLASK_SKIP_RESUME_CRAWLS").ok().and_then(|v| v.parse::<bool>().ok()).unwrap_or(false);

            if skip_resume {
                warn!("KLASK_SKIP_RESUME_CRAWLS is set - incomplete crawls will NOT be resumed on startup");
            } else {
                let service_clone = service.clone();
                tokio::spawn(async move {
                    info!("Checking for incomplete crawls to resume (in background)...");
                    if let Err(e) = service_clone.check_and_resume_incomplete_crawls().await {
                        error!("Failed to resume incomplete crawls: {}", e);
                    }
                });
            }

            // Clean up any abandoned crawls (older than 2 hours) in background
            let service_clone = service.clone();
            tokio::spawn(async move {
                if let Err(e) = service_clone.cleanup_abandoned_crawls(120).await {
                    error!("Failed to cleanup abandoned crawls: {}", e);
                }
            });

            service
        }
        Err(e) => {
            error!("Failed to initialize crawler service: {}", e);
            return Err(e);
        }
    };

    // Initialize scheduler service
    let crawler_service_arc = Arc::new(crawler_service);
    let scheduler_service = match SchedulerService::new(database.pool().clone(), crawler_service_arc.clone()).await {
        Ok(service) => {
            info!("Scheduler service initialized successfully");
            // Start the scheduler
            if let Err(e) = service.start().await {
                error!("Failed to start scheduler service: {}", e);
            } else {
                info!("Scheduler service started successfully");
            }
            service
        }
        Err(e) => {
            error!("Failed to initialize scheduler service: {}", e);
            return Err(e);
        }
    };

    // Create application state
    let app_state = AppState {
        database,
        search_service: search_service_arc,
        crawler_service: crawler_service_arc,
        progress_tracker,
        scheduler_service: Some(Arc::new(scheduler_service)),
        jwt_service,
        encryption_service,
        config: config.clone(),
        crawl_tasks: Arc::new(RwLock::new(HashMap::new())),
        startup_time,
        delete_account_rate_limiter: Arc::new(RwLock::new(HashMap::new())),
    };

    // Build application router
    let app = create_app(app_state).await?;

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;

    info!("Server listening on http://{}", bind_address);

    // Start server with graceful shutdown
    axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;

    info!("Server shutdown complete");

    Ok(())
}

/// Graceful shutdown handler that listens for SIGTERM and SIGINT signals
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C signal, initiating graceful shutdown...");
        },
        _ = terminate => {
            info!("Received SIGTERM signal, initiating graceful shutdown...");
        },
    }
}

async fn create_app(app_state: AppState) -> Result<Router> {
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/version", get(version_handler))
        .route(
            "/health",
            get({
                let db = app_state.database.clone();
                move || health_handler(db)
            }),
        )
        .nest("/api", api::create_router().await?)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    Ok(app)
}

async fn root_handler() -> String {
    format!(
        "Klask-RS: Modern Code Search Engine\n\nVersion: {}\n\nAPI: http://localhost:3000/api\nHealth: http://localhost:3000/health\nVersion Info: http://localhost:3000/version",
        version::get_version()
    )
}

async fn version_handler() -> Json<VersionInfo> {
    Json(VersionInfo {
        version: version::get_version().to_string(),
        commit: version::get_commit_hash().map(|s| s.to_string()),
        timestamp: version::get_build_timestamp().map(|s| s.to_string()),
    })
}

async fn health_handler(database: Database) -> &'static str {
    match database.health_check().await {
        Ok(_) => "OK",
        Err(_) => "Database connection failed",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_db_url() {
        // Test normal PostgreSQL URL
        let url = "postgresql://klask_user:secret_password@localhost:5432/klask_dev";
        let sanitized = sanitize_db_url(url);
        assert_eq!(sanitized, "postgresql://klask_user:****@localhost:5432/klask_dev");
        assert!(!sanitized.contains("secret_password"));

        // Test with special characters in password
        let url = "postgresql://user:p@ssw0rd!@host:5432/db";
        let sanitized = sanitize_db_url(url);
        assert_eq!(sanitized, "postgresql://user:****@host:5432/db");
        assert!(!sanitized.contains("p@ssw0rd!"));

        // Test with complex password
        let url = "postgresql://admin:My$ecret123!@db.example.com:5432/production";
        let sanitized = sanitize_db_url(url);
        assert_eq!(sanitized, "postgresql://admin:****@db.example.com:5432/production");
        assert!(!sanitized.contains("My$ecret123!"));

        // Test malformed URL (fallback to full mask)
        let url = "invalid-url";
        let sanitized = sanitize_db_url(url);
        assert_eq!(sanitized, "postgresql://****:****@****");
    }
}
