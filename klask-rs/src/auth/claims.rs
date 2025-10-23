use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub sub: Uuid, // User ID (subject)
    pub username: String,
    pub role: String,
    pub exp: i64, // Expiration time (Unix timestamp)
    pub iat: i64, // Issued at (Unix timestamp)
}

impl TokenClaims {
    pub fn new(user_id: Uuid, username: String, role: String, expires_in: Duration) -> Self {
        let now = Utc::now();
        Self { sub: user_id, username, role, exp: (now + expires_in).timestamp(), iat: now.timestamp() }
    }

    pub fn is_expired(&self) -> bool {
        Utc::now().timestamp() > self.exp
    }
}
