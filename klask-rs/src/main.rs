mod api;
mod auth;
mod config;
mod database;
mod mcp;
mod models;
mod repositories;
mod services;
mod utils;
mod version;

use anyhow::Result;
use auth::{extractors::AppState, jwt::JwtService, rate_limit::AuthRateLimits};
use axum::{
    Json, Router,
    http::{self, HeaderName, HeaderValue, StatusCode, header},
    routing::get,
};
use config::AppConfig;
use database::Database;
use serde::Serialize;
use services::{
    SearchService, crawler::CrawlerService, encryption::EncryptionService, progress::ProgressTracker,
    scheduler::SchedulerService,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::timeout::TimeoutLayer;
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
    info!("CORS allowed origins: {:?}", config.cors.allowed_origins);

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

    // Initialize the semantic-search embedding provider (optional; loads the
    // local ONNX model when enabled, degrades to keyword-only search on error)
    let semantic_embedder = services::semantic::init_embedding_provider(&config.semantic);

    // Open the vector store and start the embedding worker (optional; degrades
    // to keyword-only indexing if the store can't be opened). Only when the
    // provider loaded successfully — the indexer needs it to embed chunks.
    #[cfg(feature = "semantic-search")]
    let semantic_indexer: services::semantic::MaybeIndexer = match &semantic_embedder {
        Some(provider) => services::semantic::init_vector_indexer(&config.semantic, provider.clone()).await,
        None => None,
    };
    #[cfg(not(feature = "semantic-search"))]
    let semantic_indexer: services::semantic::MaybeIndexer = None;

    // Initialize crawler service
    let search_service_arc = Arc::new(search_service);

    // Build the semantic backfill controller so admins can rebuild the vector
    // index from the existing Tantivy documents (Phase 3). None unless both the
    // embedder and the indexer are available.
    #[cfg(feature = "semantic-search")]
    let semantic_backfill: services::semantic::MaybeBackfill = match &semantic_embedder {
        Some(provider) => {
            services::semantic::init_backfill_controller(&semantic_indexer, search_service_arc.clone(), provider)
        }
        None => None,
    };
    #[cfg(not(feature = "semantic-search"))]
    let semantic_backfill: services::semantic::MaybeBackfill = None;

    let crawler_service = match CrawlerService::new(
        database.pool().clone(),
        search_service_arc.clone(),
        progress_tracker.clone(),
        encryption_service.clone(),
        config.crawler.temp_dir.clone(),
        semantic_indexer.clone(),
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
        semantic_embedder,
        semantic_indexer,
        semantic_backfill,
        jwt_service,
        encryption_service,
        config: config.clone(),
        crawl_tasks: Arc::new(RwLock::new(HashMap::new())),
        startup_time,
        delete_account_rate_limiter: Arc::new(RwLock::new(HashMap::new())),
        auth_rate_limits: Arc::new(AuthRateLimits::default()),
    };

    // Build application router
    let app = create_app(app_state, config).await?;

    // Create TCP listener
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;

    info!("Server listening on http://{}", bind_address);

    // Start server with graceful shutdown
    // Connect info gives the peer address used by the auth rate limiter
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

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

async fn create_app(app_state: AppState, config: AppConfig) -> Result<Router> {
    // Build CORS layer with configured allowed origins
    let allowed_origins: Vec<_> =
        config.cors.allowed_origins.iter().filter_map(|origin| origin.parse::<http::HeaderValue>().ok()).collect();

    let cors = CorsLayer::new()
        .allow_origin(if allowed_origins.is_empty() {
            // Fallback: create a list with localhost if none configured
            vec![HeaderValue::from_static("http://localhost:5173"), HeaderValue::from_static("http://localhost:8080")]
        } else {
            allowed_origins
        })
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::COOKIE,
        ])
        // Required for browser clients to send HttpOnly cookies cross-origin.
        // Works because we use explicit origins (not Any).
        .allow_credentials(true);

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
        .merge(mcp::create_router())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        // Security headers for API responses. The frontend nginx adds its own
        // headers on the static site; `if_not_present` keeps this layer from
        // overriding anything set by a more specific handler.
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("cross-origin-resource-policy"),
            HeaderValue::from_static("same-origin"),
        ))
        // Bounded request bodies (413 when exceeded) and request duration (408)
        .layer(RequestBodyLimitLayer::new(config.server.max_body_bytes))
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(config.server.request_timeout_secs.max(1)),
        ))
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

/// Readiness: 200 when the database answers, 503 otherwise, so orchestrators
/// stop routing traffic to an instance that cannot serve requests.
async fn health_handler(database: Database) -> (StatusCode, &'static str) {
    match database.health_check().await {
        Ok(_) => (StatusCode::OK, "OK"),
        Err(e) => {
            error!("Health check failed: database unreachable: {}", e);
            (StatusCode::SERVICE_UNAVAILABLE, "Database connection failed")
        }
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

    #[tokio::test]
    async fn test_health_is_503_when_database_unreachable() {
        use sqlx::postgres::PgPoolOptions;
        // Nothing listens on port 1: the lazy pool cannot reach the database
        let pool = PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(2))
            .connect_lazy("postgres://klask:klask@127.0.0.1:1/klask")
            .expect("lazy pool creation does not connect");
        let (status, body) = health_handler(database::Database::from_pool(pool)).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(body, "Database connection failed");
    }
}
