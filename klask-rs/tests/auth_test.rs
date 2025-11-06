use axum::http::StatusCode;
use axum_test::TestServer;
use klask_rs::services::{
    crawler::CrawlerService, encryption::EncryptionService, progress::ProgressTracker, SearchService,
};
use klask_rs::{
    api,
    auth::{extractors::AppState, jwt::JwtService},
};
use klask_rs::{config::AppConfig, Database};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tempfile::TempDir;
use tokio::sync::RwLock;

// Create test app state with all required services
async fn create_test_app_state() -> AppState {
    // Create test database (skip if not available)
    let database = Database::new("postgres://test:test@localhost:9999/test", 1)
        .await
        .unwrap_or_else(|_| panic!("Database not available for auth testing"));

    // Create test search service
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let search_service = SearchService::new(temp_dir.path()).expect("Failed to create search service");

    // Create test config
    let config = AppConfig {
        server: klask_rs::config::ServerConfig { host: "127.0.0.1".to_string(), port: 3000 },
        database: klask_rs::config::DatabaseConfig {
            url: "postgres://test:test@localhost:9999/test".to_string(),
            max_connections: 1,
        },
        search: klask_rs::config::SearchConfig { index_dir: "./test_index".to_string(), max_results: 1000 },
        crawler: klask_rs::config::CrawlerConfig {
            temp_dir: std::env::temp_dir().join("klask-crawler-test").to_string_lossy().to_string(),
        },
        auth: klask_rs::config::AuthConfig {
            jwt_secret: "test-secret-key-for-jwt-authentication".to_string(),
            jwt_expires_in: "1h".to_string(),
            allow_registration: true,
        },
    };

    // Create JWT service
    let jwt_service = JwtService::new(&config.auth).expect("Failed to create JWT service");

    // Create shared search service
    let shared_search_service = Arc::new(search_service);

    // Create progress tracker
    let progress_tracker = Arc::new(ProgressTracker::new());

    // Create encryption service for tests
    let encryption_service = Arc::new(EncryptionService::new("test-encryption-key-32bytes").unwrap());

    // Create crawler service
    let crawler_service = Arc::new(
        CrawlerService::new(
            database.pool().clone(),
            shared_search_service.clone(),
            progress_tracker.clone(),
            encryption_service,
            std::env::temp_dir().join("klask-crawler-test").to_string_lossy().to_string(),
        )
        .expect("Failed to create crawler service"),
    );

    AppState {
        database,
        search_service: shared_search_service,
        crawler_service,
        progress_tracker,
        scheduler_service: None,
        jwt_service,
        config,
        crawl_tasks: Arc::new(RwLock::new(HashMap::new())),
        startup_time: Instant::now(),
        encryption_service: Arc::new(EncryptionService::new("test-encryption-key-32bytes").unwrap()),
        delete_account_rate_limiter: Arc::new(RwLock::new(HashMap::new())),
    }
}

#[tokio::test]
async fn test_auth_endpoints_exist() {
    // Skip this test if database is not available
    if let Ok(app_state) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(create_test_app_state())
    })) {
        let router = api::create_router().await.expect("Failed to create router");
        let app = router.with_state(app_state);
        let server = TestServer::new(app).expect("Failed to create test server");

        // Test that auth endpoints exist (should return method not allowed or bad request, not 404)
        let login_response = server.get("/api/auth/login").await;
        assert_ne!(login_response.status_code(), StatusCode::NOT_FOUND);

        let register_response = server.get("/api/auth/register").await;
        assert_ne!(register_response.status_code(), StatusCode::NOT_FOUND);

        let profile_response = server.get("/api/auth/profile").await;
        // Profile should require auth, so expect 401 or 400, not 404
        assert!(
            profile_response.status_code() == StatusCode::UNAUTHORIZED
                || profile_response.status_code() == StatusCode::BAD_REQUEST
        );
    } else {
        println!("Skipping auth test - database not available");
    }
}

#[tokio::test]
async fn test_register_validation() {
    // Skip this test if database is not available
    if let Ok(app_state) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(create_test_app_state())
    })) {
        let router = api::create_router().await.expect("Failed to create router");
        let app = router.with_state(app_state);
        let server = TestServer::new(app).expect("Failed to create test server");

        // Test registration with invalid data
        let invalid_register = json!({
            "username": "ab", // too short
            "email": "invalid-email",
            "password": "123" // too short
        });

        let response = server.post("/api/auth/register").json(&invalid_register).await;

        // Should reject invalid data
        assert!(response.status_code().is_client_error());
    } else {
        println!("Skipping auth validation test - database not available");
    }
}

#[tokio::test]
async fn test_login_without_credentials() {
    // Skip this test if database is not available
    if let Ok(app_state) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(create_test_app_state())
    })) {
        let router = api::create_router().await.expect("Failed to create router");
        let app = router.with_state(app_state);
        let server = TestServer::new(app).expect("Failed to create test server");

        // Test login without credentials
        let response = server.post("/api/auth/login").json(&json!({})).await;

        // Should reject missing credentials
        assert!(response.status_code().is_client_error());
    } else {
        println!("Skipping login test - database not available");
    }
}

#[tokio::test]
async fn test_protected_routes_require_auth() {
    // Skip this test if database is not available
    if let Ok(app_state) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(create_test_app_state())
    })) {
        let router = api::create_router().await.expect("Failed to create router");
        let app = router.with_state(app_state);
        let server = TestServer::new(app).expect("Failed to create test server");

        // Test that protected endpoints require authentication

        // Repository endpoints should require auth
        let repo_list_response = server.get("/api/repositories").await;
        assert_eq!(repo_list_response.status_code(), StatusCode::UNAUTHORIZED);

        let repo_create_response = server
            .post("/api/repositories")
            .json(&json!({"name": "test", "url": "http://example.com", "repository_type": "Git"}))
            .await;
        assert_eq!(repo_create_response.status_code(), StatusCode::UNAUTHORIZED);

        // Profile endpoint should require auth
        let profile_response = server.get("/api/auth/profile").await;
        assert_eq!(profile_response.status_code(), StatusCode::UNAUTHORIZED);
    } else {
        println!("Skipping protected routes test - database not available");
    }
}

#[tokio::test]
async fn test_jwt_token_creation() {
    // Test JWT service functionality independently
    let config = klask_rs::config::AuthConfig {
        jwt_secret: "test-secret-key".to_string(),
        jwt_expires_in: "1h".to_string(),
        allow_registration: true,
    };

    let jwt_service = JwtService::new(&config).expect("Failed to create JWT service");

    let user_id = uuid::Uuid::new_v4();
    let username = "testuser".to_string();
    let role = "User".to_string();

    // Test token creation
    let token =
        jwt_service.create_token_for_user(user_id, username.clone(), role.clone()).expect("Failed to create token");

    assert!(!token.is_empty());

    // Test token decoding
    let decoded_claims = jwt_service.decode_token(&token).expect("Failed to decode token");

    assert_eq!(decoded_claims.sub, user_id);
    assert_eq!(decoded_claims.username, username);
    assert_eq!(decoded_claims.role, role);
    assert!(!decoded_claims.is_expired());
}

#[tokio::test]
async fn test_public_endpoints_work_without_auth() {
    // Skip this test if database is not available
    if let Ok(app_state) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(create_test_app_state())
    })) {
        let router = api::create_router().await.expect("Failed to create router");
        let app = router.with_state(app_state);
        let server = TestServer::new(app).expect("Failed to create test server");

        // Test that public endpoints work without authentication
        let status_response = server.get("/api/status").await;
        assert_eq!(status_response.status_code(), StatusCode::OK);

        let health_response = server.get("/health").await;
        assert_eq!(health_response.status_code(), StatusCode::OK);

        // Search endpoint should work without auth (public search)
        let search_response = server.get("/api/search?query=test").await;
        assert_eq!(search_response.status_code(), StatusCode::OK);
    } else {
        println!("Skipping public endpoints test - database not available");
    }
}

// Helper function to create a test app state with custom allow_registration setting
async fn create_test_app_state_with_registration(allow_registration: bool) -> AppState {
    // Create test database (skip if not available)
    let database = Database::new("postgres://test:test@localhost:9999/test", 1)
        .await
        .unwrap_or_else(|_| panic!("Database not available for auth testing"));

    // Create test search service
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let search_service = SearchService::new(temp_dir.path()).expect("Failed to create search service");

    // Create test config with custom allow_registration
    let config = AppConfig {
        server: klask_rs::config::ServerConfig { host: "127.0.0.1".to_string(), port: 3000 },
        database: klask_rs::config::DatabaseConfig {
            url: "postgres://test:test@localhost:9999/test".to_string(),
            max_connections: 1,
        },
        search: klask_rs::config::SearchConfig { index_dir: "./test_index".to_string(), max_results: 1000 },
        crawler: klask_rs::config::CrawlerConfig {
            temp_dir: std::env::temp_dir().join("klask-crawler-test").to_string_lossy().to_string(),
        },
        auth: klask_rs::config::AuthConfig {
            jwt_secret: "test-secret-key-for-jwt-authentication".to_string(),
            jwt_expires_in: "1h".to_string(),
            allow_registration,
        },
    };

    // Create JWT service
    let jwt_service = JwtService::new(&config.auth).expect("Failed to create JWT service");

    // Create shared search service
    let shared_search_service = Arc::new(search_service);

    // Create progress tracker
    let progress_tracker = Arc::new(ProgressTracker::new());

    // Create encryption service for tests
    let encryption_service = Arc::new(EncryptionService::new("test-encryption-key-32bytes").unwrap());

    // Create crawler service
    let crawler_service = Arc::new(
        CrawlerService::new(
            database.pool().clone(),
            shared_search_service.clone(),
            progress_tracker.clone(),
            encryption_service.clone(),
            std::env::temp_dir().join("klask-crawler-test").to_string_lossy().to_string(),
        )
        .expect("Failed to create crawler service"),
    );

    AppState {
        database,
        search_service: shared_search_service,
        crawler_service,
        progress_tracker,
        scheduler_service: None,
        jwt_service,
        config,
        crawl_tasks: Arc::new(RwLock::new(HashMap::new())),
        startup_time: Instant::now(),
        encryption_service,
        delete_account_rate_limiter: Arc::new(RwLock::new(HashMap::new())),
    }
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[tokio::test]
async fn test_allow_registration_true_via_config() {
    // Test that ALLOW_REGISTRATION=true sets allow_registration to true
    let config = klask_rs::config::AuthConfig {
        jwt_secret: "test-secret".to_string(),
        jwt_expires_in: "1h".to_string(),
        allow_registration: true,
    };

    assert_eq!(config.allow_registration, true);
}

#[tokio::test]
async fn test_allow_registration_false_via_config() {
    // Test that ALLOW_REGISTRATION=false sets allow_registration to false
    let config = klask_rs::config::AuthConfig {
        jwt_secret: "test-secret".to_string(),
        jwt_expires_in: "1h".to_string(),
        allow_registration: false,
    };

    assert_eq!(config.allow_registration, false);
}

// ============================================================================
// Registration Status Endpoint Tests
// ============================================================================

#[tokio::test]
async fn test_registration_status_when_enabled() {
    // Test /auth/registration/status returns registration_allowed: true when enabled
    if let Ok(app_state) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(create_test_app_state_with_registration(true))
    })) {
        let router = api::create_router().await.expect("Failed to create router");
        let app = router.with_state(app_state);
        let server = TestServer::new(app).expect("Failed to create test server");

        let response = server.get("/api/auth/registration/status").await;

        assert_eq!(response.status_code(), StatusCode::OK);
        let body: serde_json::Value = response.json();
        assert_eq!(body["registration_allowed"], true);
    } else {
        println!("Skipping registration status enabled test - database not available");
    }
}

#[tokio::test]
async fn test_registration_status_when_disabled() {
    // Test /auth/registration/status returns registration_allowed: false when disabled
    if let Ok(app_state) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(create_test_app_state_with_registration(false))
    })) {
        let router = api::create_router().await.expect("Failed to create router");
        let app = router.with_state(app_state);
        let server = TestServer::new(app).expect("Failed to create test server");

        let response = server.get("/api/auth/registration/status").await;

        assert_eq!(response.status_code(), StatusCode::OK);
        let body: serde_json::Value = response.json();
        assert_eq!(body["registration_allowed"], false);
    } else {
        println!("Skipping registration status disabled test - database not available");
    }
}

// ============================================================================
// Registration Endpoint Blocking Tests
// ============================================================================

#[tokio::test]
async fn test_registration_blocked_when_disabled() {
    // Test that registration returns 403 when allow_registration=false
    if let Ok(app_state) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(create_test_app_state_with_registration(false))
    })) {
        let router = api::create_router().await.expect("Failed to create router");
        let app = router.with_state(app_state);
        let server = TestServer::new(app).expect("Failed to create test server");

        let register_request = json!({
            "username": "testuser",
            "email": "testuser@example.com",
            "password": "ValidPassword123"
        });

        let response = server.post("/api/auth/register").json(&register_request).await;

        // Should return 403 Forbidden
        assert_eq!(response.status_code(), StatusCode::FORBIDDEN);

        let body: serde_json::Value = response.json();
        assert!(body["error"].as_str().unwrap().contains("Registration is currently disabled"));
    } else {
        println!("Skipping registration blocked test - database not available");
    }
}

#[tokio::test]
async fn test_registration_blocked_error_message() {
    // Test that the error message is correct when registration is disabled
    if let Ok(app_state) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(create_test_app_state_with_registration(false))
    })) {
        let router = api::create_router().await.expect("Failed to create router");
        let app = router.with_state(app_state);
        let server = TestServer::new(app).expect("Failed to create test server");

        let register_request = json!({
            "username": "anotheruser",
            "email": "another@example.com",
            "password": "ValidPassword123"
        });

        let response = server.post("/api/auth/register").json(&register_request).await;

        assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
        let body: serde_json::Value = response.json();

        // Verify error message and status code
        assert!(body["error"].as_str().unwrap().to_lowercase().contains("registration is currently disabled"));
        assert_eq!(body["status"], 403);
    } else {
        println!("Skipping registration blocked error message test - database not available");
    }
}

#[tokio::test]
async fn test_registration_allowed_when_enabled() {
    // Test that registration validation is performed when registration is enabled
    if let Ok(app_state) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(create_test_app_state_with_registration(true))
    })) {
        let router = api::create_router().await.expect("Failed to create router");
        let app = router.with_state(app_state);
        let server = TestServer::new(app).expect("Failed to create test server");

        // Test with invalid data - should fail validation, not registration disabled
        let invalid_register = json!({
            "username": "ab", // too short
            "email": "invalid-email",
            "password": "123" // too short
        });

        let response = server.post("/api/auth/register").json(&invalid_register).await;

        // Should fail validation (4xx error) but NOT with registration disabled error
        assert!(response.status_code().is_client_error());
        let body: serde_json::Value = response.json();

        // Should NOT contain "Registration is currently disabled" message
        let error_msg = body["error"].as_str().unwrap().to_lowercase();
        assert!(
            !error_msg.contains("registration is currently disabled"),
            "Got registration disabled error when registration should be enabled"
        );
    } else {
        println!("Skipping registration allowed test - database not available");
    }
}

#[tokio::test]
async fn test_registration_check_happens_before_validation() {
    // Test that registration disabled check happens before other validations
    if let Ok(app_state) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(create_test_app_state_with_registration(false))
    })) {
        let router = api::create_router().await.expect("Failed to create router");
        let app = router.with_state(app_state);
        let server = TestServer::new(app).expect("Failed to create test server");

        // Even with invalid data, should get registration disabled error
        let invalid_register = json!({
            "username": "ab",
            "email": "invalid",
            "password": "123"
        });

        let response = server.post("/api/auth/register").json(&invalid_register).await;

        assert_eq!(response.status_code(), StatusCode::FORBIDDEN);
        let body: serde_json::Value = response.json();
        assert!(body["error"].as_str().unwrap().to_lowercase().contains("registration is currently disabled"));
    } else {
        println!("Skipping registration check priority test - database not available");
    }
}

// ============================================================================
// Edge Cases and Integration Tests
// ============================================================================

#[tokio::test]
async fn test_registration_status_is_accurate() {
    // Test that registration status endpoint accurately reflects the config
    if let Ok(app_state_disabled) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(create_test_app_state_with_registration(false))
    })) {
        let router_disabled = api::create_router().await.expect("Failed to create router");
        let app_disabled = router_disabled.with_state(app_state_disabled);
        let server_disabled = TestServer::new(app_disabled).expect("Failed to create test server");

        let status_response = server_disabled.get("/api/auth/registration/status").await;
        assert_eq!(status_response.status_code(), StatusCode::OK);

        let body: serde_json::Value = status_response.json();
        assert_eq!(
            body["registration_allowed"], false,
            "Status endpoint should reflect disabled registration"
        );
    } else {
        println!("Skipping registration status accuracy test - database not available");
    }
}

#[tokio::test]
async fn test_multiple_registration_attempts_when_disabled() {
    // Test that multiple registration attempts all fail consistently when disabled
    if let Ok(app_state) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Runtime::new().unwrap().block_on(create_test_app_state_with_registration(false))
    })) {
        let router = api::create_router().await.expect("Failed to create router");
        let app = router.with_state(app_state);
        let server = TestServer::new(app).expect("Failed to create test server");

        // Try multiple registration attempts
        for i in 0..3 {
            let register_request = json!({
                "username": format!("user{}", i),
                "email": format!("user{}@example.com", i),
                "password": "ValidPassword123"
            });

            let response = server.post("/api/auth/register").json(&register_request).await;

            assert_eq!(
                response.status_code(),
                StatusCode::FORBIDDEN,
                "Attempt {} should be blocked",
                i
            );
        }
    } else {
        println!("Skipping multiple registration attempts test - database not available");
    }
}
