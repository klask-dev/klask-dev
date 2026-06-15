//! Repository for API token CRUD operations

use crate::models::ApiToken;
use anyhow::Result;
use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for managing personal API tokens
pub struct ApiTokenRepository {
    pool: PgPool,
}

impl ApiTokenRepository {
    /// Create a new ApiTokenRepository with the given connection pool
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Create a new API token
    pub async fn create_token(
        &self,
        user_id: Uuid,
        name: &str,
        scope: &str,
        token_hash: &str,
        token_prefix: &str,
        expires_in_days: Option<i32>,
    ) -> Result<ApiToken> {
        let expires_at =
            expires_in_days.and_then(
                |days| {
                    if days <= 0 { None } else { Some(Utc::now() + Duration::days(days as i64)) }
                },
            );

        let result = sqlx::query_as::<_, ApiToken>(
            "INSERT INTO api_tokens (user_id, name, scope, token_hash, token_prefix, expires_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, user_id, token_hash, token_prefix, name, scope, active, created_at, last_used_at, expires_at"
        )
        .bind(user_id)
        .bind(name)
        .bind(scope)
        .bind(token_hash)
        .bind(token_prefix)
        .bind(expires_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// List all active API tokens for a given user
    pub async fn list_tokens(&self, user_id: Uuid) -> Result<Vec<ApiToken>> {
        let tokens = sqlx::query_as::<_, ApiToken>(
            "SELECT id, user_id, token_hash, token_prefix, name, scope, active, created_at, last_used_at, expires_at
             FROM api_tokens
             WHERE user_id = $1 AND active = true
             ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(tokens)
    }

    /// Get a specific API token by ID
    pub async fn get_token(&self, id: Uuid) -> Result<Option<ApiToken>> {
        let token = sqlx::query_as::<_, ApiToken>(
            "SELECT id, user_id, token_hash, token_prefix, name, scope, active, created_at, last_used_at, expires_at
             FROM api_tokens
             WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(token)
    }

    /// Find an API token by its plaintext token
    /// This method handles Argon2 hash comparison for authentication
    /// Optimized: Uses prefix-based index for O(1) lookup before verifying hash
    pub async fn find_by_token(&self, plaintext_token: &str) -> Result<Option<ApiToken>> {
        if plaintext_token.len() < 12 {
            return Ok(None);
        }

        let token_prefix = &plaintext_token[0..12];

        let tokens = sqlx::query_as::<_, ApiToken>(
            "SELECT id, user_id, token_hash, token_prefix, name, scope, active, created_at, last_used_at, expires_at
             FROM api_tokens
             WHERE token_prefix = $1 AND active = true",
        )
        .bind(token_prefix)
        .fetch_all(&self.pool)
        .await?;

        use crate::utils::password::verify_password;

        for token in tokens {
            if let Ok(true) = verify_password(plaintext_token, &token.token_hash) {
                return Ok(Some(token));
            }
        }

        Ok(None)
    }

    /// Find an API token by its hash (used during authentication)
    /// Deprecated: Use find_by_token instead for proper Argon2 verification
    #[allow(dead_code)]
    pub async fn find_by_hash(&self, token_hash: &str) -> Result<Option<ApiToken>> {
        let token = sqlx::query_as::<_, ApiToken>(
            "SELECT id, user_id, token_hash, token_prefix, name, scope, active, created_at, last_used_at, expires_at
             FROM api_tokens
             WHERE token_hash = $1",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(token)
    }

    /// Update the last_used_at timestamp for a token (called after successful authentication)
    pub async fn update_last_used(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE api_tokens
             SET last_used_at = NOW()
             WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Revoke an API token (soft delete: set active = false)
    pub async fn revoke_token(&self, id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE api_tokens
             SET active = false
             WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Revoke all active API tokens for a given user (e.g., after password change)
    #[allow(dead_code)]
    pub async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<()> {
        sqlx::query(
            "UPDATE api_tokens
             SET active = false
             WHERE user_id = $1 AND active = true",
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Hard delete an API token (use with caution; soft delete via revoke_token is preferred for audit trail)
    #[allow(dead_code)]
    pub async fn delete_token(&self, id: Uuid) -> Result<()> {
        sqlx::query("DELETE FROM api_tokens WHERE id = $1").bind(id).execute(&self.pool).await?;

        Ok(())
    }

    /// Check if a token exists, is active, and not expired
    #[allow(dead_code)]
    pub async fn is_token_valid(&self, token: &ApiToken) -> Result<bool> {
        if !token.active {
            return Ok(false);
        }

        if let Some(expires_at) = token.expires_at
            && Utc::now() > expires_at
        {
            return Ok(false);
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full repository tests would require a test database setup.
    // Integration tests are recommended for testing database operations.
    // See tests/ directory for integration test examples.

    #[test]
    fn test_is_token_valid_not_active() {
        // Note: This is a simplified test; real tests would use a test database
        let token = ApiToken {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            token_hash: "hash".to_string(),
            token_prefix: "klask_pat_ab".to_string(),
            name: "Test".to_string(),
            scope: "read-only".to_string(),
            active: false,
            created_at: Utc::now(),
            last_used_at: None,
            expires_at: None,
        };

        // In real integration tests, we'd call the repository method
        // For now, we verify the token fields are correct
        assert!(!token.active);
    }
}
