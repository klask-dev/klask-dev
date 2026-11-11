use crate::auth::claims::TokenClaims;
use crate::config::AuthConfig;
use anyhow::Result;
use chrono::Duration;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};

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
        let expires_in = Duration::seconds(crate::config::parse_duration_secs(&config.jwt_expires_in)?);

        let mut validation = Validation::default();
        validation.set_issuer(&["klask"]);
        validation.set_audience(&["klask"]);

        Ok(Self {
            encoding_key: EncodingKey::from_secret(secret),
            decoding_key: DecodingKey::from_secret(secret),
            validation,
            expires_in,
        })
    }

    /// Session lifetime in seconds; also used for the cookie `Max-Age`.
    pub fn expires_in_secs(&self) -> i64 {
        self.expires_in.num_seconds()
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
}
