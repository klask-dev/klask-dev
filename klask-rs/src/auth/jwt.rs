use crate::auth::claims::TokenClaims;
use crate::config::AuthConfig;
use anyhow::Result;
use chrono::Duration;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

#[derive(Clone)]
pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
    expires_in: Duration,
}

impl JwtService {
    pub fn new(config: &AuthConfig) -> Result<Self> {
        let secret = config.jwt_secret.as_bytes();

        // Parse expires_in from config (e.g., "24h", "1d", "60m")
        let expires_in = Self::parse_duration(&config.jwt_expires_in)?;

        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            validation: Validation::default(),
            expires_in,
        })
    }

    pub fn encode_token(&self, claims: &TokenClaims) -> Result<String> {
        encode(&Header::default(), claims, &self.encoding_key)
            .map_err(|e| anyhow::anyhow!("Failed to encode JWT: {}", e))
    }

    pub fn decode_token(&self, token: &str) -> Result<TokenClaims> {
        decode::<TokenClaims>(token, &self.decoding_key, &self.validation)
            .map(|data| data.claims)
            .map_err(|e| anyhow::anyhow!("Failed to decode JWT: {}", e))
    }

    pub fn create_token_for_user(&self, user_id: uuid::Uuid, username: String, role: String) -> Result<String> {
        let claims = TokenClaims::new(user_id, username, role, self.expires_in);
        self.encode_token(&claims)
    }

    fn parse_duration(duration_str: &str) -> Result<Duration> {
        if duration_str.ends_with('h') {
            let hours: i64 = duration_str.trim_end_matches('h').parse()?;
            Ok(Duration::hours(hours))
        } else if duration_str.ends_with('d') {
            let days: i64 = duration_str.trim_end_matches('d').parse()?;
            Ok(Duration::days(days))
        } else if duration_str.ends_with('m') {
            let minutes: i64 = duration_str.trim_end_matches('m').parse()?;
            Ok(Duration::minutes(minutes))
        } else if duration_str.ends_with('s') {
            let seconds: i64 = duration_str.trim_end_matches('s').parse()?;
            Ok(Duration::seconds(seconds))
        } else {
            // Default to hours if no unit specified
            let hours: i64 = duration_str.parse()?;
            Ok(Duration::hours(hours))
        }
    }
}
