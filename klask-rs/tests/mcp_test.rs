//! Integration tests for the MCP (Model Context Protocol) server endpoint.
//!
//! Covers the Streamable HTTP handshake, tool listing, tool execution against
//! a real Tantivy index, authentication and JSON-RPC error handling.

use anyhow::Result;
use axum::http::StatusCode;
use axum_test::TestServer;
use klask_rs::{
    auth::{extractors::AppState, jwt::JwtService},
    config::{AppConfig, AuthConfig, CorsConfig, CrawlerConfig, DatabaseConfig, SearchConfig, ServerConfig},
    database::Database,
    models::UserRole,
    services::{
        FileData, SearchService, crawler::CrawlerService, encryption::EncryptionService, progress::ProgressTracker,
    },
};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc, time::Instant};
use tempfile::TempDir;
use tokio::sync::RwLock;
use uuid::Uuid;

const TEST_JWT_SECRET: &str = "mcp-test-secret-0123456789abcdef0123456789abcdef";

async fn setup_test_server() -> Result<(TestServer, AppState, TempDir)> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://klask_user:klask_password@localhost/klask_test".to_string());

    let database = Database::new(&database_url, 5).await?;

    let temp_dir = TempDir::new()?;
    let index_dir = temp_dir.path().join("index");
    let search_service = Arc::new(SearchService::new(&index_dir)?);

    let config = AppConfig {
        server: ServerConfig { host: "127.0.0.1".to_string(), port: 0 },
        database: DatabaseConfig { url: database_url, max_connections: 5 },
        search: SearchConfig { index_dir: index_dir.to_string_lossy().to_string(), max_results: 10000 },
        crawler: CrawlerConfig { temp_dir: temp_dir.path().join("crawler").to_string_lossy().to_string() },
        auth: AuthConfig {
            jwt_secret: TEST_JWT_SECRET.to_string(),
            jwt_expires_in: "1h".to_string(),
            allow_registration: true,
        },
        cors: CorsConfig { allowed_origins: vec![] },
        semantic: klask_rs::config::SemanticSearchConfig {
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

    let progress_tracker = Arc::new(ProgressTracker::new());
    let jwt_service = JwtService::new(&config.auth)?;
    let encryption_service = Arc::new(EncryptionService::new("test-encryption-key-32bytes")?);
    let crawler_service = Arc::new(CrawlerService::new(
        database.pool().clone(),
        search_service.clone(),
        progress_tracker.clone(),
        encryption_service.clone(),
        config.crawler.temp_dir.clone(),
        None,
    )?);

    let app_state = AppState {
        database,
        search_service,
        crawler_service,
        progress_tracker,
        scheduler_service: None,
        semantic_embedder: None,
        semantic_indexer: None,
        jwt_service,
        encryption_service,
        config,
        crawl_tasks: Arc::new(RwLock::new(HashMap::new())),
        startup_time: Instant::now(),
        delete_account_rate_limiter: Arc::new(RwLock::new(HashMap::new())),
        login_rate_limiter: Arc::new(RwLock::new(HashMap::new())),
    };

    let app = klask_rs::mcp::create_router().with_state(app_state.clone());
    let server = TestServer::new(app);

    Ok((server, app_state, temp_dir))
}

/// Create a regular user (unique per test, tests share the database) and a valid token.
async fn create_user_token(app_state: &AppState) -> Result<String> {
    let user_id = Uuid::new_v4();
    let username = format!("mcp_user_{}", &user_id.to_string()[..8]);
    let now = chrono::Utc::now();

    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, role, active, created_at, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(user_id)
    .bind(&username)
    .bind(format!("{username}@example.com"))
    .bind("test_hash")
    .bind(UserRole::User)
    .bind(true)
    .bind(now)
    .bind(now)
    .execute(app_state.database.pool())
    .await?;

    let token = app_state.jwt_service.create_token_for_user(user_id, username, UserRole::User.to_string())?;
    Ok(token)
}

/// Index a small fixture file and commit so it is searchable.
async fn index_fixture_file(app_state: &AppState) -> Result<Uuid> {
    let file_id = Uuid::new_v4();
    let content =
        "fn validate_jwt_token(token: &str) -> bool {\n    // verify signature and expiry\n    !token.is_empty()\n}\n";

    app_state
        .search_service
        .index_file(FileData {
            file_id,
            file_name: "auth.rs",
            file_path: "src/auth.rs",
            content,
            repository: "klask",
            project: "klask",
            version: "main",
            extension: "rs",
            size: content.len() as u64,
        })
        .await?;
    app_state.search_service.force_flush().await?;

    Ok(file_id)
}

async fn rpc(server: &TestServer, token: &str, body: &Value) -> axum_test::TestResponse {
    server.post("/mcp").add_header("Authorization", &format!("Bearer {token}")).json(body).await
}

/// Extract and parse the JSON payload of a tools/call text result.
fn tool_payload(response_body: &Value) -> Value {
    let result = &response_body["result"];
    assert_eq!(result["isError"], false, "tool call should succeed: {result}");
    let text = result["content"][0]["text"].as_str().expect("text content");
    serde_json::from_str(text).expect("tool payload should be valid JSON")
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_mcp_initialize_handshake() -> Result<()> {
    let (server, app_state, _tmp) = setup_test_server().await?;
    let token = create_user_token(&app_state).await?;

    let response = rpc(
        &server,
        &token,
        &json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "test-client", "version": "0.0.1" }
            }
        }),
    )
    .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    assert_eq!(body["jsonrpc"], "2.0");
    assert_eq!(body["id"], 1);
    assert_eq!(body["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(body["result"]["serverInfo"]["name"], "klask");
    assert!(body["result"]["capabilities"]["tools"].is_object());

    // The follow-up notification must be accepted without a JSON-RPC response
    let response = rpc(
        &server,
        &token,
        &json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
    )
    .await;
    assert_eq!(response.status_code(), StatusCode::ACCEPTED);

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_mcp_tools_list() -> Result<()> {
    let (server, app_state, _tmp) = setup_test_server().await?;
    let token = create_user_token(&app_state).await?;

    let response = rpc(
        &server,
        &token,
        &json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
    )
    .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body: Value = response.json();
    let tools = body["result"]["tools"].as_array().expect("tools array");
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
    assert_eq!(
        names,
        vec!["search_code", "get_file", "list_repositories", "get_search_facets"]
    );

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_mcp_search_and_get_file_roundtrip() -> Result<()> {
    let (server, app_state, _tmp) = setup_test_server().await?;
    let token = create_user_token(&app_state).await?;
    index_fixture_file(&app_state).await?;

    // Search for the fixture content
    let response = rpc(
        &server,
        &token,
        &json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "search_code",
                "arguments": { "query": "validate_jwt_token", "extensions": ["rs"] }
            }
        }),
    )
    .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let payload = tool_payload(&response.json());
    assert!(
        payload["total"].as_u64().unwrap() >= 1,
        "expected at least one hit: {payload}"
    );
    let first = &payload["results"][0];
    assert_eq!(first["path"], "src/auth.rs");
    assert_eq!(first["version"], "main");
    let doc_address = first["doc_address"].as_str().expect("doc_address").to_string();

    // Fetch the full file via its doc_address
    let response = rpc(
        &server,
        &token,
        &json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "get_file",
                "arguments": { "doc_address": doc_address }
            }
        }),
    )
    .await;

    let payload = tool_payload(&response.json());
    assert_eq!(payload["path"], "src/auth.rs");
    assert_eq!(payload["truncated"], false);
    assert!(payload["content"].as_str().unwrap().contains("validate_jwt_token"));

    // Fetch a single line range, this time via file_id
    let response = rpc(
        &server,
        &token,
        &json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "tools/call",
            "params": {
                "name": "get_file",
                "arguments": { "file_id": payload["file_id"], "start_line": 2, "end_line": 2 }
            }
        }),
    )
    .await;
    let payload = tool_payload(&response.json());
    assert_eq!(payload["start_line"], 2);
    assert_eq!(payload["end_line"], 2);
    assert!(payload["content"].as_str().unwrap().contains("verify signature"));

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_mcp_get_search_facets() -> Result<()> {
    let (server, app_state, _tmp) = setup_test_server().await?;
    let token = create_user_token(&app_state).await?;
    index_fixture_file(&app_state).await?;

    let response = rpc(
        &server,
        &token,
        &json!({
            "jsonrpc": "2.0",
            "id": 7,
            "method": "tools/call",
            "params": { "name": "get_search_facets" }
        }),
    )
    .await;

    let payload = tool_payload(&response.json());
    let extensions = payload["extensions"].as_array().expect("extensions facet");
    assert!(
        extensions.iter().any(|f| f["value"] == "rs" && f["count"].as_u64().unwrap() >= 1),
        "expected an rs facet: {payload}"
    );

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_mcp_list_repositories_empty() -> Result<()> {
    let (server, app_state, _tmp) = setup_test_server().await?;
    let token = create_user_token(&app_state).await?;

    let response = rpc(
        &server,
        &token,
        &json!({
            "jsonrpc": "2.0",
            "id": 8,
            "method": "tools/call",
            "params": { "name": "list_repositories" }
        }),
    )
    .await;

    let payload = tool_payload(&response.json());
    let page_len = payload["repositories"].as_array().expect("repositories array").len();
    let limit = payload["limit"].as_u64().unwrap() as usize;
    assert_eq!(payload["page"], 1);
    assert!(page_len <= limit);
    assert!(payload["total"].as_u64().unwrap() as usize >= page_len);

    // Disabled repositories are excluded by default, so every returned repo is enabled
    for repo in payload["repositories"].as_array().unwrap() {
        assert_eq!(repo["enabled"], true, "unexpected disabled repo: {repo}");
    }

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_mcp_requires_authentication() -> Result<()> {
    let (server, _app_state, _tmp) = setup_test_server().await?;

    let response = server.post("/mcp").json(&json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"})).await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_mcp_protocol_errors() -> Result<()> {
    let (server, app_state, _tmp) = setup_test_server().await?;
    let token = create_user_token(&app_state).await?;

    // Unknown method -> -32601
    let response = rpc(
        &server,
        &token,
        &json!({"jsonrpc": "2.0", "id": 1, "method": "does/not/exist"}),
    )
    .await;
    let body: Value = response.json();
    assert_eq!(body["error"]["code"], -32601);

    // Malformed JSON -> -32700
    let response = server
        .post("/mcp")
        .add_header("Authorization", &format!("Bearer {token}"))
        .add_header("Content-Type", "application/json")
        .text("{not json")
        .await;
    let body: Value = response.json();
    assert_eq!(body["error"]["code"], -32700);

    // Well-formed JSON that is not a valid Request object -> -32600, echoing the id
    let response = rpc(&server, &token, &json!({"jsonrpc": "2.0", "id": 9})).await;
    let body: Value = response.json();
    assert_eq!(body["error"]["code"], -32600);
    assert_eq!(body["id"], 9);

    // Unknown tool -> -32602
    let response = rpc(
        &server,
        &token,
        &json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/call",
            "params": { "name": "rm_dash_rf", "arguments": {} }
        }),
    )
    .await;
    let body: Value = response.json();
    assert_eq!(body["error"]["code"], -32602);

    // Missing required argument -> -32602
    let response = rpc(
        &server,
        &token,
        &json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": { "name": "search_code", "arguments": {} }
        }),
    )
    .await;
    let body: Value = response.json();
    assert_eq!(body["error"]["code"], -32602);

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_mcp_get_method_not_allowed() -> Result<()> {
    let (server, app_state, _tmp) = setup_test_server().await?;
    let token = create_user_token(&app_state).await?;

    let response = server.get("/mcp").add_header("Authorization", &format!("Bearer {token}")).await;
    assert_eq!(response.status_code(), StatusCode::METHOD_NOT_ALLOWED);

    Ok(())
}
