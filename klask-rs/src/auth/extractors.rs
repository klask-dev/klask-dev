use crate::auth::{claims::TokenClaims, errors::AuthError, jwt::JwtService};
use crate::database::Database;
use crate::models::user::{User, UserRole};
use crate::repositories::user_repository::UserRepository;
use crate::services::{encryption::EncryptionService, progress::ProgressTracker};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{error, trace, warn};
use uuid::Uuid;

// Application state that will be shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub search_service: Arc<crate::services::SearchService>,
    pub crawler_service: Arc<crate::services::crawler::CrawlerService>,
    pub progress_tracker: Arc<ProgressTracker>,
    pub scheduler_service: Option<Arc<crate::services::scheduler::SchedulerService>>,
    /// Embedding provider for semantic search; None when disabled or not
    /// compiled in. Used by the query path (embed the query text) coming in
    /// Phase 4 of docs/SEMANTIC_SEARCH_PLAN.md.
    #[allow(dead_code)]
    pub semantic_embedder: Option<Arc<dyn crate::services::semantic::EmbeddingProvider>>,
    /// Background embedding worker that mirrors crawled files into the vector
    /// store. None when semantic search is disabled or not compiled in.
    #[allow(dead_code)]
    pub semantic_indexer: crate::services::semantic::MaybeIndexer,
    /// Controller for the admin "rebuild semantic index" backfill job (Phase 3).
    /// None when semantic search is disabled or not compiled in.
    #[allow(dead_code)]
    pub semantic_backfill: crate::services::semantic::MaybeBackfill,
    pub jwt_service: JwtService,
    pub encryption_service: Arc<EncryptionService>,
    #[allow(dead_code)]
    pub config: crate::config::AppConfig,
    #[allow(dead_code)]
    pub crawl_tasks: Arc<RwLock<HashMap<Uuid, tokio::task::JoinHandle<()>>>>,
    pub startup_time: Instant,
    /// Rate limiter for delete account attempts (user_id -> (attempts, last_reset_time))
    pub delete_account_rate_limiter: Arc<RwLock<HashMap<Uuid, (u32, std::time::SystemTime)>>>,
    /// Rate limiter for login attempts (username -> (attempts, last_reset_time))
    pub login_rate_limiter: Arc<RwLock<HashMap<String, (u32, std::time::SystemTime)>>>,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub user: User,
    #[allow(dead_code)]
    pub claims: TokenClaims,
}

impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let token = extract_token(&parts.headers)?;
        extract_authenticated_user(state, &token).await
    }
}

// Role-based authentication extractor for admin users
#[derive(Debug, Clone)]
pub struct AdminUser(pub AuthenticatedUser);

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        trace!("Attempting to extract AdminUser from request");

        let token = extract_token(&parts.headers)?;
        let auth_user = extract_authenticated_user(state, &token).await?;

        if auth_user.user.role != UserRole::Admin {
            warn!(
                "User {} attempted to access admin endpoint without admin role",
                auth_user.user.username
            );
            return Err(AuthError::InsufficientPermissions);
        }

        trace!("AdminUser extracted successfully for user: {}", auth_user.user.username);
        Ok(AdminUser(auth_user))
    }
}

/// Extract a JWT token from the request.
/// Priority:
///   1. `Authorization: Bearer <token>` header  (API clients / backward compat)
///   2. `auth_token` HttpOnly cookie             (browser clients)
fn extract_token(headers: &axum::http::HeaderMap) -> Result<String, AuthError> {
    // 1. Try Authorization header first (API clients)
    if let Ok(token) = extract_token_from_auth_header(headers) {
        return Ok(token);
    }

    // 2. Fallback: read from HttpOnly cookie (browser clients)
    let cookies = headers.get("cookie").and_then(|v| v.to_str().ok()).unwrap_or("");

    extract_token_from_cookie(cookies, "auth_token").ok_or(AuthError::MissingAuthHeader)
}

fn extract_token_from_auth_header(headers: &axum::http::HeaderMap) -> Result<String, AuthError> {
    let auth_header = headers
        .get("authorization")
        .ok_or(AuthError::MissingAuthHeader)?
        .to_str()
        .map_err(|_| AuthError::InvalidAuthHeader)?;

    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        Ok(token.to_string())
    } else {
        Err(AuthError::InvalidAuthHeader)
    }
}

/// Parse a cookie string like "a=1; auth_token=XYZ; b=2" and return the value for `name`.
fn extract_token_from_cookie(cookies: &str, name: &str) -> Option<String> {
    for part in cookies.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix(name)
            && let Some(value) = value.strip_prefix('=')
        {
            let token = value.trim().to_string();
            if !token.is_empty() {
                return Some(token);
            }
        }
    }
    None
}

/// Helper function to extract and validate an authenticated user from a token
async fn extract_authenticated_user(state: &AppState, token: &str) -> Result<AuthenticatedUser, AuthError> {
    trace!("Extracting AuthenticatedUser from request");

    // Decode and validate token
    let claims = state.jwt_service.decode_token(token).map_err(|e| {
        error!("Failed to decode token: {:?}", e);
        AuthError::InvalidToken(e.to_string())
    })?;

    trace!("Token decoded successfully for user ID: {}", claims.sub);

    // Check if token is expired
    if claims.is_expired() {
        warn!("Token expired for user ID: {}", claims.sub);
        return Err(AuthError::TokenExpired);
    }

    // Fetch user from database
    let user_repo = UserRepository::new(state.database.pool().clone());
    let user = user_repo
        .get_user(claims.sub)
        .await
        .map_err(|e| {
            error!("Database error while fetching user {}: {:?}", claims.sub, e);
            AuthError::DatabaseError(e.to_string())
        })?
        .ok_or_else(|| {
            warn!("User not found for ID: {}", claims.sub);
            AuthError::UserNotFound
        })?;

    trace!("User found: {}", user.username);

    // Verify user is active
    if !user.active {
        warn!("Inactive user attempted to authenticate: {}", user.username);
        return Err(AuthError::UserInactive);
    }

    if claims.iat < user.password_changed_at.timestamp() {
        warn!("Token issued before password change for user: {}", user.username);
        return Err(AuthError::TokenExpired);
    }

    trace!("AuthenticatedUser extracted successfully: {}", user.username);
    Ok(AuthenticatedUser { user, claims })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token_from_cookie() {
        // Basic case
        assert_eq!(
            extract_token_from_cookie("auth_token=abc123", "auth_token"),
            Some("abc123".to_string())
        );

        // Multiple cookies
        assert_eq!(
            extract_token_from_cookie("session=xyz; auth_token=mytoken; other=val", "auth_token"),
            Some("mytoken".to_string())
        );

        // Missing cookie
        assert_eq!(extract_token_from_cookie("session=xyz; other=val", "auth_token"), None);

        // Empty value
        assert_eq!(extract_token_from_cookie("auth_token=", "auth_token"), None);
    }
}
