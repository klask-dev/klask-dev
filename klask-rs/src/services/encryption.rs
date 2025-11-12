use aes_gcm::{
    Aes256Gcm,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use anyhow::Result;
use base64::{Engine as _, engine::general_purpose};
use sqlx::Row;

pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
    /// Create a new encryption service from ENCRYPTION_KEY environment variable
    /// with validation against database tokens
    pub async fn new_from_env(pool: &sqlx::PgPool) -> Result<Self> {
        use tracing::{error, info};

        // Validate ENCRYPTION_KEY environment variable
        let encryption_key = match std::env::var("ENCRYPTION_KEY") {
            Ok(key) => {
                // Check 1: Key is not empty
                if key.is_empty() {
                    error!("ENCRYPTION_KEY environment variable is empty. Please provide a non-empty encryption key.");
                    return Err(anyhow::anyhow!("ENCRYPTION_KEY is empty"));
                }

                // Check 2: Key meets minimum length
                if key.len() < 16 {
                    error!(
                        "ENCRYPTION_KEY must be at least 16 characters long. Current length: {}",
                        key.len()
                    );
                    return Err(anyhow::anyhow!("ENCRYPTION_KEY is too short (minimum 16 characters)"));
                }
                key
            }
            Err(_) => {
                // Check 3: Key variable is defined
                error!("ENCRYPTION_KEY environment variable is not set. This is required for secure token storage.");
                error!("Set ENCRYPTION_KEY to a random string of at least 16 characters.");
                error!("Generate one with: openssl rand -hex 32");
                return Err(anyhow::anyhow!("ENCRYPTION_KEY environment variable not set"));
            }
        };

        // Create the service with the validated key
        let service = Self::new(&encryption_key)?;

        // Validate encryption service against database tokens
        service.validate_with_database(pool).await?;

        info!("Encryption service initialized and validated successfully");
        Ok(service)
    }

    /// Create a new encryption service with a key from environment or config
    pub fn new(key_string: &str) -> Result<Self> {
        // The key should be 32 bytes for AES-256
        let key_bytes = if key_string.len() == 32 {
            key_string.as_bytes().to_vec()
        } else {
            // Hash the key to get exactly 32 bytes
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(key_string.as_bytes());
            hasher.finalize().to_vec()
        };

        // Create key from slice
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|_| anyhow::anyhow!("Invalid key length - must be 32 bytes"))?;

        Ok(Self { cipher })
    }

    /// Encrypt a token or sensitive data
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        // Generate a random nonce (96 bits for AES-GCM)
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        // Encrypt the plaintext
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;

        // Combine nonce and ciphertext
        let mut combined = nonce.to_vec();
        combined.extend_from_slice(&ciphertext);

        // Encode as base64 for storage
        Ok(general_purpose::STANDARD.encode(combined))
    }

    /// Decrypt a token or sensitive data
    pub fn decrypt(&self, encrypted: &str) -> Result<String> {
        // Decode from base64
        let combined = general_purpose::STANDARD
            .decode(encrypted)
            .map_err(|e| anyhow::anyhow!("Failed to decode base64: {:?}", e))?;

        // Split nonce and ciphertext
        if combined.len() < 12 {
            return Err(anyhow::anyhow!("Invalid encrypted data"));
        }

        let (nonce_bytes, ciphertext) = combined.split_at(12);
        // Create nonce from slice using try_into - aes_gcm's Nonce can be created from [u8; 12]
        let nonce_array: [u8; 12] = nonce_bytes.try_into().map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;

        // Decrypt
        let plaintext = self
            .cipher
            .decrypt((&nonce_array).into(), ciphertext)
            .map_err(|e| anyhow::anyhow!("Decryption failed: {:?}", e))?;

        String::from_utf8(plaintext).map_err(|e| anyhow::anyhow!("Failed to convert decrypted data to string: {:?}", e))
    }

    /// Validate that the encryption service can decrypt existing tokens in the database
    /// This ensures the ENCRYPTION_KEY hasn't changed since the tokens were encrypted
    /// If there are no tokens, performs a basic roundtrip test instead
    pub async fn validate_with_database(&self, pool: &sqlx::PgPool) -> Result<()> {
        use tracing::{error, info};

        // Try to find a repository with an encrypted access token
        let result = sqlx::query("SELECT id, access_token FROM repositories WHERE access_token IS NOT NULL LIMIT 1")
            .fetch_optional(pool)
            .await;

        match result {
            Ok(Some(row)) => {
                // Found an encrypted token in the database, try to decrypt it
                info!("Found encrypted token in database, validating ENCRYPTION_KEY...");

                let encrypted_token: Option<String> = row.get("access_token");
                let encrypted_token = encrypted_token.ok_or_else(|| anyhow::anyhow!("Token field is null"))?;

                match self.decrypt(&encrypted_token) {
                    Ok(decrypted) => {
                        if decrypted.is_empty() {
                            return Err(anyhow::anyhow!(
                                "ENCRYPTION_KEY validation failed: decrypted token is empty"
                            ));
                        }
                        info!("Successfully decrypted existing token from database - ENCRYPTION_KEY is valid");
                        Ok(())
                    }
                    Err(e) => {
                        error!("ENCRYPTION_KEY validation FAILED: Cannot decrypt existing tokens in database");
                        error!("This likely means the ENCRYPTION_KEY has changed since the tokens were encrypted");
                        error!(
                            "Please restore the correct ENCRYPTION_KEY or clear the encrypted tokens from the database"
                        );
                        Err(anyhow::anyhow!(
                            "Failed to decrypt existing token: {}. ENCRYPTION_KEY may have changed.",
                            e
                        ))
                    }
                }
            }
            Ok(None) => {
                // No encrypted tokens in database, perform a basic roundtrip test
                info!("No existing tokens in database, performing basic encryption validation...");

                const TEST_DATA: &str = "encryption_key_validation_test";

                let encrypted =
                    self.encrypt(TEST_DATA).map_err(|e| anyhow::anyhow!("Encryption validation failed: {}", e))?;

                let decrypted =
                    self.decrypt(&encrypted).map_err(|e| anyhow::anyhow!("Decryption validation failed: {}", e))?;

                if decrypted != TEST_DATA {
                    return Err(anyhow::anyhow!(
                        "Encryption/decryption roundtrip failed: decrypted data does not match original"
                    ));
                }

                info!("Basic encryption validation passed");
                Ok(())
            }
            Err(e) => {
                error!("Failed to query database for token validation: {}", e);
                Err(anyhow::anyhow!(
                    "Database query failed during ENCRYPTION_KEY validation: {}",
                    e
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_decryption() {
        let service = EncryptionService::new("my-secret-encryption-key-32bytes").unwrap();

        let original = "my-secret-token";
        let encrypted = service.encrypt(original).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(original, decrypted);
        assert_ne!(original, encrypted);
    }
}
