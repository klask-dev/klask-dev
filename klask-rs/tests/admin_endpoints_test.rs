use anyhow::Result;
use axum::http::StatusCode;
use axum_test::TestServer;
use klask_rs::{
    api::admin::{AdminDashboardData, SystemStats},
    auth::{claims::TokenClaims, extractors::AppState, jwt::JwtService},
    config::AppConfig,
    database::Database,
    models::{User, UserRole},
    services::seeding::SeedingStats,
    services::{
        crawler::CrawlerService, encryption::EncryptionService, progress::ProgressTracker, search::SearchService,
    },
};
use serde_json::Value;
use sqlx::PgPool;
use std::{collections::HashMap, sync::Arc, time::Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

// Test utilities
async fn setup_test_server() -> Result<(TestServer, AppState)> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@localhost/klask_test".to_string());

    let database = Database::new(&database_url, 5).await?;

    // Clean up any existing test data
    cleanup_test_data(database.pool()).await?;

    let config = AppConfig::default();
    let search_service = SearchService::new("./test_index")?;

    // Create required services for AppState
    let progress_tracker = Arc::new(ProgressTracker::new());
    let jwt_service = JwtService::new(&config.auth).expect("Failed to create JWT service");
    let encryption_service = Arc::new(EncryptionService::new("test-encryption-key-32bytes").unwrap());
    let crawler_service = Arc::new(
        CrawlerService::new(
            database.pool().clone(),
            Arc::new(search_service.clone()),
            progress_tracker.clone(),
            encryption_service,
            std::env::temp_dir().join("klask-crawler-test").to_string_lossy().to_string(),
        )
        .expect("Failed to create crawler service"),
    );

    let app_state = AppState {
        database,
        search_service: Arc::new(search_service),
        crawler_service,
        progress_tracker,
        scheduler_service: None,
        jwt_service,
        config,
        crawl_tasks: Arc::new(RwLock::new(HashMap::new())),
        startup_time: Instant::now(),
        encryption_service: Arc::new(EncryptionService::new("test-encryption-key-32bytes").unwrap()),
    };

    let app = klask_rs::api::create_router().await?.with_state(app_state.clone());
    let server = TestServer::new(app)?;

    Ok((server, app_state))
}

async fn cleanup_test_data(pool: &PgPool) -> Result<()> {
    // Files are now only in Tantivy search index, no database table to clean
    sqlx::query("DELETE FROM repositories").execute(pool).await?;
    sqlx::query("DELETE FROM users").execute(pool).await?;
    Ok(())
}

async fn create_admin_token(app_state: &AppState) -> Result<String> {
    let admin_user = User {
        id: Uuid::new_v4(),
        username: "test_admin".to_string(),
        email: "test_admin@example.com".to_string(),
        password_hash: "test_hash".to_string(),
        role: UserRole::Admin,
        active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_login: None,
        last_activity: None,
    };

    // Insert admin user
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, role, active, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(admin_user.id)
    .bind(&admin_user.username)
    .bind(&admin_user.email)
    .bind(&admin_user.password_hash)
    .bind(&admin_user.role)
    .bind(admin_user.active)
    .bind(admin_user.created_at)
    .bind(admin_user.updated_at)
    .execute(app_state.database.pool())
    .await?;

    let claims = TokenClaims {
        sub: admin_user.id,
        username: admin_user.username,
        role: admin_user.role.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret("test_secret".as_ref()),
    )?;

    Ok(token)
}

async fn create_regular_user_token(app_state: &AppState) -> Result<String> {
    let user = User {
        id: Uuid::new_v4(),
        username: "test_user".to_string(),
        email: "test_user@example.com".to_string(),
        password_hash: "test_hash".to_string(),
        role: UserRole::User,
        active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        last_login: None,
        last_activity: None,
    };

    // Insert regular user
    sqlx::query(
        "INSERT INTO users (id, username, email, password_hash, role, active, created_at, updated_at) 
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(user.id)
    .bind(&user.username)
    .bind(&user.email)
    .bind(&user.password_hash)
    .bind(&user.role)
    .bind(user.active)
    .bind(user.created_at)
    .bind(user.updated_at)
    .execute(app_state.database.pool())
    .await?;

    let claims = TokenClaims {
        sub: user.id,
        username: user.username,
        role: user.role.to_string(),
        exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        iat: chrono::Utc::now().timestamp(),
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret("test_secret".as_ref()),
    )?;

    Ok(token)
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_admin_dashboard_endpoint_requires_admin() -> Result<()> {
    let (server, app_state) = setup_test_server().await?;

    // Test without authentication
    let response = server.get("/api/admin/dashboard").await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);

    // Test with regular user token
    let user_token = create_regular_user_token(&app_state).await?;
    let response =
        server.get("/api/admin/dashboard").add_header("Authorization", &format!("Bearer {}", user_token)).await;
    assert_eq!(response.status_code(), StatusCode::FORBIDDEN);

    // Test with admin token
    let admin_token = create_admin_token(&app_state).await?;
    let response =
        server.get("/api/admin/dashboard").add_header("Authorization", &format!("Bearer {}", admin_token)).await;
    assert_eq!(response.status_code(), StatusCode::OK);

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_admin_dashboard_returns_complete_data() -> Result<()> {
    let (server, app_state) = setup_test_server().await?;
    let admin_token = create_admin_token(&app_state).await?;

    let response =
        server.get("/api/admin/dashboard").add_header("Authorization", &format!("Bearer {}", admin_token)).await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let dashboard_data: AdminDashboardData = response.json();

    // Verify all required sections are present
    assert!(!dashboard_data.system.version.is_empty());
    assert!(!dashboard_data.system.environment.is_empty());
    assert!(
        dashboard_data.system.database_status == "Connected" || dashboard_data.system.database_status == "Disconnected"
    );
    // uptime_seconds is u64, so it's always >= 0
    let _ = dashboard_data.system.uptime_seconds;

    // User stats should include at least our test admin user
    assert!(dashboard_data.users.total_users >= 1);

    // Repository and content stats should be initialized (even if empty)
    assert!(dashboard_data.repositories.total_repositories >= 0);
    assert!(dashboard_data.content.total_files >= 0);
    assert!(dashboard_data.search.total_documents >= 0);

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_system_stats_endpoint() -> Result<()> {
    let (server, app_state) = setup_test_server().await?;
    let admin_token = create_admin_token(&app_state).await?;

    let response =
        server.get("/api/admin/system").add_header("Authorization", &format!("Bearer {}", admin_token)).await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let system_stats: SystemStats = response.json();

    assert!(!system_stats.version.is_empty());
    assert!(!system_stats.environment.is_empty());
    // uptime_seconds is u64, so it's always >= 0
    let _ = system_stats.uptime_seconds;
    assert!(system_stats.database_status == "Connected" || system_stats.database_status == "Disconnected");

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_seed_database_endpoint() -> Result<()> {
    let (server, app_state) = setup_test_server().await?;
    let admin_token = create_admin_token(&app_state).await?;

    // Get initial stats
    let initial_response =
        server.get("/api/admin/seed/stats").add_header("Authorization", &format!("Bearer {}", admin_token)).await;
    assert_eq!(initial_response.status_code(), StatusCode::OK);

    let initial_stats: SeedingStats = initial_response.json();

    // Seed the database
    let seed_response =
        server.post("/api/admin/seed").add_header("Authorization", &format!("Bearer {}", admin_token)).await;

    assert_eq!(seed_response.status_code(), StatusCode::OK);

    let seed_result: SeedingStats = seed_response.json();

    // Verify data was created
    assert!(seed_result.users_created > initial_stats.users_created);
    assert!(seed_result.repositories_created > initial_stats.repositories_created);

    // Expected counts based on seeding service
    assert_eq!(seed_result.users_created, 5);
    assert_eq!(seed_result.repositories_created, 3);

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_clear_seed_data_endpoint() -> Result<()> {
    let (server, app_state) = setup_test_server().await?;
    let admin_token = create_admin_token(&app_state).await?;

    // First seed the database
    let seed_response =
        server.post("/api/admin/seed").add_header("Authorization", &format!("Bearer {}", admin_token)).await;
    assert_eq!(seed_response.status_code(), StatusCode::OK);

    // Verify data exists
    let stats_response =
        server.get("/api/admin/seed/stats").add_header("Authorization", &format!("Bearer {}", admin_token)).await;
    let stats: SeedingStats = stats_response.json();
    assert!(stats.users_created > 0);
    assert!(stats.repositories_created > 0);

    // Clear seed data
    let clear_response =
        server.post("/api/admin/seed/clear").add_header("Authorization", &format!("Bearer {}", admin_token)).await;
    assert_eq!(clear_response.status_code(), StatusCode::OK);

    let clear_result: SeedingStats = clear_response.json();

    // Verify data was cleared (but admin user should still exist)
    assert_eq!(clear_result.users_created, 0);
    assert_eq!(clear_result.repositories_created, 0);

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_seed_stats_endpoint() -> Result<()> {
    let (server, app_state) = setup_test_server().await?;
    let admin_token = create_admin_token(&app_state).await?;

    let response =
        server.get("/api/admin/seed/stats").add_header("Authorization", &format!("Bearer {}", admin_token)).await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let stats: SeedingStats = response.json();

    // Should return valid statistics (may be zero if no seed data)
    assert!(stats.users_created >= 0);
    assert!(stats.repositories_created >= 0);

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_user_stats_endpoint() -> Result<()> {
    let (server, app_state) = setup_test_server().await?;
    let admin_token = create_admin_token(&app_state).await?;

    let response =
        server.get("/api/admin/users/stats").add_header("Authorization", &format!("Bearer {}", admin_token)).await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let stats: Value = response.json();

    // Should have basic user stats structure
    assert!(stats.get("total_users").is_some());
    assert!(stats.get("active_users").is_some());
    assert!(stats.get("admin_users").is_some());

    // Should include at least our test admin user
    let total_users = stats["total_users"].as_i64().unwrap();
    assert!(total_users >= 1);

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_repository_stats_endpoint() -> Result<()> {
    let (server, app_state) = setup_test_server().await?;
    let admin_token = create_admin_token(&app_state).await?;

    let response = server
        .get("/api/admin/repositories/stats")
        .add_header("Authorization", &format!("Bearer {}", admin_token))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let stats: Value = response.json();

    // Should have repository stats structure
    assert!(stats.get("total_repositories").is_some());
    assert!(stats.get("enabled_repositories").is_some());
    assert!(stats.get("disabled_repositories").is_some());

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_content_stats_endpoint() -> Result<()> {
    let (server, app_state) = setup_test_server().await?;
    let admin_token = create_admin_token(&app_state).await?;

    let response =
        server.get("/api/admin/content/stats").add_header("Authorization", &format!("Bearer {}", admin_token)).await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let stats: Value = response.json();

    // Should have content stats structure
    assert!(stats.get("total_files").is_some());
    assert!(stats.get("total_size_bytes").is_some());
    assert!(stats.get("files_by_extension").is_some());
    assert!(stats.get("files_by_project").is_some());

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_search_stats_endpoint() -> Result<()> {
    let (server, app_state) = setup_test_server().await?;
    let admin_token = create_admin_token(&app_state).await?;

    let response =
        server.get("/api/admin/search/stats").add_header("Authorization", &format!("Bearer {}", admin_token)).await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let stats: Value = response.json();

    // Should have search stats structure
    assert!(stats.get("total_documents").is_some());
    assert!(stats.get("index_size_mb").is_some());

    let total_documents = stats["total_documents"].as_i64().unwrap();
    let index_size_mb = stats["index_size_mb"].as_f64().unwrap();

    assert!(total_documents >= 0);
    assert!(index_size_mb >= 0.0);

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_recent_activity_endpoint() -> Result<()> {
    let (server, app_state) = setup_test_server().await?;
    let admin_token = create_admin_token(&app_state).await?;

    let response =
        server.get("/api/admin/activity/recent").add_header("Authorization", &format!("Bearer {}", admin_token)).await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let activity: Value = response.json();

    // Should have recent activity structure
    assert!(activity.get("recent_users").is_some());
    assert!(activity.get("recent_repositories").is_some());
    assert!(activity.get("recent_crawls").is_some());

    let recent_users = activity["recent_users"].as_array().unwrap();

    // Should include our test admin user if created recently
    assert!(!recent_users.is_empty());

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_error_handling_invalid_tokens() -> Result<()> {
    let (server, _app_state) = setup_test_server().await?;

    // Test with invalid token format
    let response = server.get("/api/admin/dashboard").add_header("Authorization", "Bearer invalid_token").await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);

    // Test with malformed authorization header
    let response = server.get("/api/admin/dashboard").add_header("Authorization", "InvalidFormat").await;
    assert_eq!(response.status_code(), StatusCode::UNAUTHORIZED);

    Ok(())
}

#[tokio::test]
#[ignore = "Requires PostgreSQL database"]
async fn test_concurrent_admin_requests() -> Result<()> {
    let (server, app_state) = setup_test_server().await?;
    let admin_token = create_admin_token(&app_state).await?;

    // Make multiple concurrent requests to different endpoints
    let dashboard_future =
        server.get("/api/admin/dashboard").add_header("Authorization", &format!("Bearer {}", admin_token.clone()));

    let system_future =
        server.get("/api/admin/system").add_header("Authorization", &format!("Bearer {}", admin_token.clone()));

    let stats_future =
        server.get("/api/admin/seed/stats").add_header("Authorization", &format!("Bearer {}", admin_token));

    let (dashboard_response, system_response, stats_response) =
        tokio::join!(dashboard_future, system_future, stats_future);

    // All requests should succeed
    assert_eq!(dashboard_response.status_code(), StatusCode::OK);
    assert_eq!(system_response.status_code(), StatusCode::OK);
    assert_eq!(stats_response.status_code(), StatusCode::OK);

    Ok(())
}
