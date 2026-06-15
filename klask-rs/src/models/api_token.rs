//! Personal API tokens for programmatic access to Klask
//!
//! Tokens follow the format: `klask_pat_` (10 chars) + 32 random alphanumeric characters.
//! Tokens are hashed using SHA-256 before being stored in the database.
//! The plaintext token is shown to the user only once during creation.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use sqlx::FromRow;
use uuid::Uuid;

/// Represents a stored API token (internal use, includes hash)
#[derive(Debug, Serialize, Deserialize, Clone, FromRow)]
pub struct ApiToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub token_prefix: String,
    pub name: String,
    pub scope: String,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Public representation of an API token (safe to return in API responses)
/// This struct excludes the token hash for security
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiTokenInfo {
    pub id: Uuid,
    pub token_prefix: String,
    pub name: String,
    pub scope: String,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Response sent when a new API token is created
/// This is the only time the plaintext token is revealed to the user
#[derive(Debug, Serialize)]
pub struct CreateApiTokenResponse {
    pub id: Uuid,
    pub token: String, // Plaintext token (shown only once)
    pub token_prefix: String,
    pub name: String,
    pub scope: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Request to create a new API token
#[derive(Debug, Deserialize)]
pub struct CreateApiTokenRequest {
    pub name: String,
    #[serde(default)]
    pub expires_in_days: Option<i32>,
}

impl From<ApiToken> for ApiTokenInfo {
    fn from(token: ApiToken) -> Self {
        Self {
            id: token.id,
            token_prefix: token.token_prefix,
            name: token.name,
            scope: token.scope,
            active: token.active,
            created_at: token.created_at,
            last_used_at: token.last_used_at,
            expires_at: token.expires_at,
        }
    }
}

/// Generate a new personal API token with the format: `klask_pat_` + 32 random alphanumeric characters
///
/// Total length: 42 characters
/// Example: `klask_pat_a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6`
pub fn generate_api_token() -> String {
    use rand::RngExt;

    const PREFIX: &str = "klask_pat_";
    const RANDOM_CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    const RANDOM_LEN: usize = 32;

    let mut rng = rand::rng();
    let random_part: String = (0..RANDOM_LEN)
        .map(|_| {
            let idx = rng.random_range(0..RANDOM_CHARS.len());
            RANDOM_CHARS[idx] as char
        })
        .collect();

    format!("{}{}", PREFIX, random_part)
}

/// Extract the display-safe prefix from a full token
/// The prefix is used in the database to allow listing tokens without exposing the full hash
///
/// # Panics
/// Panics if the token is shorter than 12 characters (expected format)
pub fn extract_token_prefix(token: &str) -> String {
    // klask_pat_ = 10 chars, + 2 chars from the random part = 12 total
    token[0..12].to_string()
}

/// Hash an API token using SHA-256
/// This is fast and suitable for short-lived, revocable tokens
pub fn hash_api_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_api_token() {
        let token = generate_api_token();
        assert_eq!(token.len(), 42);
        assert!(token.starts_with("klask_pat_"));
    }

    #[test]
    fn test_generate_api_token_uniqueness() {
        let token1 = generate_api_token();
        let token2 = generate_api_token();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_extract_token_prefix() {
        let token = "klask_pat_abcdefghij";
        let prefix = extract_token_prefix(token);
        assert_eq!(prefix, "klask_pat_ab");
    }

    #[test]
    fn test_api_token_to_info_conversion() {
        let now = chrono::Utc::now();
        let token = ApiToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            token_hash: "hash".to_string(),
            token_prefix: "klask_pat_ab".to_string(),
            name: "Test Token".to_string(),
            scope: "read-only".to_string(),
            active: true,
            created_at: now,
            last_used_at: None,
            expires_at: None,
        };

        let info: ApiTokenInfo = token.into();
        assert_eq!(info.name, "Test Token");
        assert!(!info.token_prefix.contains("hash")); // Hash should not be in ApiTokenInfo
    }
}
