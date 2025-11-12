mod api;
mod auth;
mod config;
mod database;
mod models;
mod repositories;
mod services;
mod utils;

use anyhow::Result;
use auth::{extractors::AppState, jwt::JwtService};
use axum::{Router, routing::get};
use config::AppConfig;
use database::Database;
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
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Validate that the encryption service can decrypt existing tokens in the database
/// This ensures the ENCRYPTION_KEY hasn't changed since the tokens were encrypted
/// If there are no tokens, performs a basic roundtrip test instead
async fn validate_encryption_service(service: &EncryptionService, database: &Database) -> Result<()> {
    use sqlx::Row;

    // Try to find a repository with an encrypted access token
    let result = sqlx::query("SELECT id, access_token FROM repositories WHERE access_token IS NOT NULL LIMIT 1")
        .fetch_optional(database.pool())
        .await;

    match result {
        Ok(Some(row)) => {
            // Found an encrypted token in the database, try to decrypt it
            info!("Found encrypted token in database, validating ENCRYPTION_KEY...");

            let encrypted_token: Option<String> = row.get("access_token");
            let encrypted_token = encrypted_token.ok_or_else(|| anyhow::anyhow!("Token field is null"))?;

            match service.decrypt(&encrypted_token) {
                Ok(decrypted) => {
                    if decrypted.is_empty() {
                        return Err(anyhow::anyhow!(
                            "ENCRYPTION_KEY validation failed: decrypted token is empty"
                        ));
                    }
                    info!("Successfully decrypted existing token from database - ENCRYPTION_KEY is valid");
                    Ok(())
                }
                Err(e) => {
                    error!("ENCRYPTION_KEY validation FAILED: Cannot decrypt existing tokens in database");
                    error!("This likely means the ENCRYPTION_KEY has changed since the tokens were encrypted");
                    error!("Please restore the correct ENCRYPTION_KEY or clear the encrypted tokens from the database");
                    Err(anyhow::anyhow!(
                        "Failed to decrypt existing token: {}. ENCRYPTION_KEY may have changed.",
                        e
                    ))
                }
            }
        }
        Ok(None) => {
            // No encrypted tokens in database, perform a basic roundtrip test
            info!("No existing tokens in database, performing basic encryption validation...");

            const TEST_DATA: &str = "encryption_key_validation_test";

            let encrypted =
                service.encrypt(TEST_DATA).map_err(|e| anyhow::anyhow!("Encryption validation failed: {}", e))?;

            let decrypted =
                service.decrypt(&encrypted).map_err(|e| anyhow::anyhow!("Decryption validation failed: {}", e))?;

            if decrypted != TEST_DATA {
                return Err(anyhow::anyhow!(
                    "Encryption/decryption roundtrip failed: decrypted data does not match original"
                ));
            }

            info!("Basic encryption validation passed");
            Ok(())
        }
        Err(e) => {
            error!("Failed to query database for token validation: {}", e);
            Err(anyhow::anyhow!(
                "Database query failed during ENCRYPTION_KEY validation: {}",
                e
            ))
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    // Build the filter with quiet modules first, then apply RUST_LOG or defaults
    let rust_log = std::env::var("RUST_LOG")
        .unwrap_or_else(|_| "klask_rs=debug,tower_http=debug,tantivy=info,sqlx=warn".to_string());
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
            error!("Failed to connect to database: {}", e);
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
    let encryption_key = match std::env::var("ENCRYPTION_KEY") {
        Ok(key) => {
            if key.is_empty() {
                error!("ENCRYPTION_KEY environment variable is empty. Please provide a non-empty encryption key.");
                return Err(anyhow::anyhow!("ENCRYPTION_KEY is empty"));
            }
            if key.len() < 16 {
                error!(
                    "ENCRYPTION_KEY must be at least 16 characters long. Current length: {}",
                    key.len()
                );
                return Err(anyhow::anyhow!("ENCRYPTION_KEY is too short (minimum 16 characters)"));
            }
            key
        }
        Err(_) => {
            error!("ENCRYPTION_KEY environment variable is not set. This is required for secure token storage.");
            error!("Set ENCRYPTION_KEY to a random string of at least 16 characters.");
            error!("Generate one with: openssl rand -hex 32");
            return Err(anyhow::anyhow!("ENCRYPTION_KEY environment variable not set"));
        }
    };

    let encryption_service = match EncryptionService::new(&encryption_key) {
        Ok(service) => {
            // Validate encryption service against database tokens
            match validate_encryption_service(&service, &database).await {
                Ok(_) => {
                    info!("Encryption service initialized and validated successfully");
                    Arc::new(service)
                }
                Err(e) => {
                    error!("Encryption service validation failed: {}", e);
                    return Err(e);
                }
            }
        }
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
    ) {
        Ok(service) => {
            info!("Crawler service initialized successfully");

            // Check for incomplete crawls and resume them in background
            // This must not block server startup
            let service_clone = service.clone();
            tokio::spawn(async move {
                info!("Checking for incomplete crawls to resume (in background)...");
                if let Err(e) = service_clone.check_and_resume_incomplete_crawls().await {
                    error!("Failed to resume incomplete crawls: {}", e);
                }
            });

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

async fn root_handler() -> &'static str {
    "Klask-RS: Modern Code Search Engine"
}

async fn health_handler(database: Database) -> &'static str {
    match database.health_check().await {
        Ok(_) => "OK",
        Err(_) => "Database connection failed",
    }
}
