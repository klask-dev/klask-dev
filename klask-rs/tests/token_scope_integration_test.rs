//! Integration tests for API token scopes and their revocation on password change.
//!
//! These tests need a PostgreSQL database and are therefore `#[ignore]`d. Run them with:
//!
//! ```text
//! DATABASE_URL="postgres://klask_user:klask_password@localhost/klask_test" \
//!   cargo test --test token_scope_integration_test -- --ignored --test-threads=2
//! ```
//!
//! The database is shared: each test uses unique usernames and never truncates
//! tables. `DATABASE_URL` must NOT point at a development database; the tests
//! refuse to run when the URL names `klask_dev`.

use axum::Router;
use axum::http::{StatusCode, header};
use axum_test::TestServer;
use klask_rs::auth::extractors::AppState;
use klask_rs::auth::jwt::JwtService;
use klask_rs::config::{
    AppConfig, AuthConfig, CorsConfig, CrawlerConfig, DatabaseConfig, RepositoryPolicyConfig, SearchConfig,
    SemanticSearchConfig, ServerConfig,
};
use klask_rs::models::{TokenScope, User, UserRole, extract_token_prefix, generate_api_token, hash_api_token};
use klask_rs::repositories::{ApiTokenRepository, UserRepository};
use klask_rs::services::{
    SearchService, crawler::CrawlerService, encryption::EncryptionService, progress::ProgressTracker,
};
use klask_rs::utils::password::hash_password;
use klask_rs::{Database, api, mcp};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;
use tokio::sync::RwLock;
use uuid::Uuid;

const OLD_PASSWORD: &str = "OldPassword1";
const NEW_PASSWORD: &str = "NewPassword2";

/// Build the application state against the database named by `DATABASE_URL`.
async fn build_state() -> (AppState, TempDir) {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for this ignored test");
    assert!(
        !database_url.contains("klask_dev"),
        "refusing to run destructive-capable tests against a development database"
    );

    let database = Database::new(&database_url, 4).await.expect("connect to test database");
    let index_dir = TempDir::new().expect("temp index dir");
    let search_service = SearchService::new(index_dir.path()).expect("search service");
    let shared_search = Arc::new(search_service);
    let progress_tracker = Arc::new(ProgressTracker::new());
    let encryption_service = Arc::new(EncryptionService::new("test-encryption-key-32bytes").expect("encryption"));
    let crawler_service = Arc::new(
        CrawlerService::new(
            database.pool().clone(),
            shared_search.clone(),
            progress_tracker.clone(),
            encryption_service.clone(),
            std::env::temp_dir().join("klask-crawler-scope-test").to_string_lossy().to_string(),
            None,
        )
        .expect("crawler service"),
    );

    let config = AppConfig {
        server: ServerConfig {
            host: "127.0.0.1".to_string(),
            port: 0,
            max_body_bytes: 4_194_304,
            request_timeout_secs: 60,
        },
        database: DatabaseConfig { url: database_url, max_connections: 4 },
        search: SearchConfig { index_dir: index_dir.path().to_string_lossy().to_string(), max_results: 1000 },
        crawler: CrawlerConfig {
            temp_dir: std::env::temp_dir().join("klask-crawler-scope-test").to_string_lossy().to_string(),
        },
        auth: AuthConfig {
            jwt_secret: "test-secret-key-for-scope-integration-tests".to_string(),
            jwt_expires_in: "1h".to_string(),
            allow_registration: false,
            cookie_secure: false,
            trusted_proxy_hops: 0,
            setup_token: None,
        },
        cors: CorsConfig { allowed_origins: vec!["http://localhost:5173".to_string()] },
        repositories: RepositoryPolicyConfig { filesystem_allowed_roots: vec![], allow_private_network_urls: true },
        semantic: SemanticSearchConfig {
            enabled: false,
            model: "jinaai/jina-embeddings-v2-base-code".to_string(),
            cache_dir: "./models".to_string(),
            vector_store_dir: "./vector-index".to_string(),
            chunk_max_lines: 60,
            chunk_overlap_lines: 15,
            batch_size: 32,
            queue_capacity: 1000,
        },
    };

    let jwt_service = JwtService::new(&config.auth).expect("jwt service");
    let state = AppState {
        database,
        search_service: shared_search,
        crawler_service,
        progress_tracker,
        scheduler_service: None,
        semantic_embedder: None,
        semantic_indexer: None,
        semantic_backfill: None,
        jwt_service,
        encryption_service,
        config,
        crawl_tasks: Arc::new(RwLock::new(HashMap::new())),
        startup_time: Instant::now(),
        delete_account_rate_limiter: Arc::new(RwLock::new(HashMap::new())),
        auth_rate_limits: Arc::new(klask_rs::auth::rate_limit::AuthRateLimits::default()),
    };
    (state, index_dir)
}

/// Same composition as `main.rs`: API under `/api`, MCP at `/mcp`.
async fn build_server(state: AppState) -> TestServer {
    let router: Router = Router::new()
        .nest("/api", api::create_router().await.expect("api router"))
        .merge(mcp::create_router())
        .with_state(state);
    TestServer::new(router)
}

/// Insert a user with a unique name and return it.
async fn create_user(state: &AppState, role: UserRole) -> User {
    let suffix = Uuid::new_v4().simple().to_string();
    let now = chrono::Utc::now();
    let user = User {
        id: Uuid::new_v4(),
        username: format!("scope_{}", &suffix[..12]),
        email: format!("scope_{}@example.test", &suffix[..12]),
        password_hash: hash_password(OLD_PASSWORD).expect("hash"),
        role,
        active: true,
        created_at: now,
        updated_at: now,
        last_login: None,
        last_activity: None,
        avatar_url: None,
        bio: None,
        full_name: None,
        phone: None,
        timezone: Some("UTC".to_string()),
        preferences: None,
        login_count: 0,
        password_changed_at: now,
    };
    UserRepository::new(state.database.pool().clone()).create_user(&user).await.expect("create user")
}

/// Create a personal API token with the given scope; returns the plaintext token.
async fn create_api_token(state: &AppState, user: &User, scope: TokenScope) -> String {
    let plaintext = generate_api_token();
    ApiTokenRepository::new(state.database.pool().clone())
        .create_token(
            user.id,
            "integration test token",
            scope.as_str(),
            &hash_api_token(&plaintext),
            &extract_token_prefix(&plaintext),
            None,
        )
        .await
        .expect("create api token");
    plaintext
}

fn bearer(token: &str) -> String {
    format!("Bearer {}", token)
}

#[tokio::test]
#[ignore = "requires PostgreSQL (DATABASE_URL pointing at klask_test)"]
async fn read_only_token_is_refused_on_writes_and_admin_routes() {
    let (state, _index) = build_state().await;
    let admin = create_user(&state, UserRole::Admin).await;
    let admin_read_only = create_api_token(&state, &admin, TokenScope::ReadOnly).await;
    let server = build_server(state).await;

    // Admin route: refused although the owner is an administrator
    let response = server.get("/api/users").add_header(header::AUTHORIZATION, bearer(&admin_read_only)).await;
    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);

    // Write route for a regular authenticated endpoint
    let response = server
        .put("/api/auth/profile")
        .add_header(header::AUTHORIZATION, bearer(&admin_read_only))
        .json(&serde_json::json!({"bio": "changed by a read-only token"}))
        .await;
    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
}

#[tokio::test]
#[ignore = "requires PostgreSQL (DATABASE_URL pointing at klask_test)"]
async fn read_only_token_is_accepted_for_reads_and_mcp() {
    let (state, _index) = build_state().await;
    let user = create_user(&state, UserRole::User).await;
    let token = create_api_token(&state, &user, TokenScope::ReadOnly).await;
    let server = build_server(state).await;

    let response = server.get("/api/auth/profile").add_header(header::AUTHORIZATION, bearer(&token)).await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let response = server
        .post("/mcp")
        .add_header(header::AUTHORIZATION, bearer(&token))
        .json(&serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "ping"}))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "requires PostgreSQL (DATABASE_URL pointing at klask_test)"]
async fn read_write_token_is_accepted_for_writes() {
    let (state, _index) = build_state().await;
    let user = create_user(&state, UserRole::User).await;
    let token = create_api_token(&state, &user, TokenScope::ReadWrite).await;
    let server = build_server(state).await;

    let response = server
        .put("/api/auth/profile")
        .add_header(header::AUTHORIZATION, bearer(&token))
        .json(&serde_json::json!({"bio": "changed by a read-write token"}))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "requires PostgreSQL (DATABASE_URL pointing at klask_test)"]
async fn read_write_admin_token_reaches_admin_routes() {
    let (state, _index) = build_state().await;
    let admin = create_user(&state, UserRole::Admin).await;
    let token = create_api_token(&state, &admin, TokenScope::ReadWrite).await;
    let server = build_server(state).await;

    let response = server.get("/api/users").add_header(header::AUTHORIZATION, bearer(&token)).await;
    assert_eq!(response.status_code(), StatusCode::OK);
}

#[tokio::test]
#[ignore = "requires PostgreSQL (DATABASE_URL pointing at klask_test)"]
async fn password_change_revokes_api_tokens() {
    let (state, _index) = build_state().await;
    let user = create_user(&state, UserRole::User).await;
    let token = create_api_token(&state, &user, TokenScope::ReadWrite).await;
    let server = build_server(state.clone()).await;

    // Token works before the change
    let response = server.get("/api/auth/profile").add_header(header::AUTHORIZATION, bearer(&token)).await;
    assert_eq!(response.status_code(), StatusCode::OK);

    // Change the password (the session cookie is not needed: the token authenticates the request)
    let response = server
        .put("/api/auth/password")
        .add_header(header::AUTHORIZATION, bearer(&token))
        .json(&serde_json::json!({
            "current_password": OLD_PASSWORD,
            "new_password": NEW_PASSWORD,
            "new_password_confirm": NEW_PASSWORD,
        }))
        .await;
    assert_eq!(response.status_code(), StatusCode::OK);

    // The token is revoked
    let response = server.get("/api/auth/profile").add_header(header::AUTHORIZATION, bearer(&token)).await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);

    // Stored as inactive
    let tokens =
        ApiTokenRepository::new(state.database.pool().clone()).list_tokens(user.id).await.expect("list tokens");
    assert!(
        tokens.iter().all(|t| !t.active),
        "all tokens of the account must be revoked"
    );
}
