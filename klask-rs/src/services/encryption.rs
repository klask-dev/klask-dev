use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm,
};
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};

pub struct EncryptionService {
    cipher: Aes256Gcm,
}

impl EncryptionService {
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
