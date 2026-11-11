//! Password hashing and verification utilities using Argon2id

use anyhow::Result;
use argon2::{
    Argon2, Params,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

/// Create an Argon2 instance with balanced security and performance parameters
/// - Memory: 19MB (balance between security and speed)
/// - Time cost: 2 iterations
/// - Parallelism: 1 thread
fn create_argon2() -> Argon2<'static> {
    Argon2::new(
        argon2::Algorithm::default(),
        argon2::Version::default(),
        Params::new(19 * 1024, 2, 1, Some(Params::DEFAULT_OUTPUT_LEN)).unwrap_or_default(),
    )
}

/// Hash a password using Argon2id with secure parameters
pub fn hash_password(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = create_argon2();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Password hashing failed: {}", e))?
        .to_string();
    Ok(password_hash)
}

/// Hash a password on the blocking thread pool.
///
/// Argon2 is deliberately expensive (tens of milliseconds of CPU); running it
/// inline in an async handler stalls the Tokio worker that polls every other
/// request scheduled on it.
pub async fn hash_password_async(password: String) -> Result<String> {
    tokio::task::spawn_blocking(move || hash_password(&password))
        .await
        .map_err(|e| anyhow::anyhow!("Password hashing task failed: {}", e))?
}

/// Verify a password against an Argon2id hash on the blocking thread pool.
/// See [`hash_password_async`] for why.
pub async fn verify_password_async(password: String, hash: String) -> Result<bool> {
    tokio::task::spawn_blocking(move || verify_password(&password, &hash))
        .await
        .map_err(|e| anyhow::anyhow!("Password verification task failed: {}", e))?
}

/// Verify a password against an Argon2id hash
pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
    let parsed_hash = PasswordHash::new(hash).map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;

    let argon2 = create_argon2();
    match argon2.verify_password(password.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(anyhow::anyhow!("Password verification error: {}", e)),
    }
}

/// Get a dummy hash for timing attack mitigation
/// Returns a valid Argon2id hash that will always fail verification
/// Used to equalize response times when a user is not found
pub fn get_dummy_hash() -> &'static str {
    // A static valid Argon2id hash that doesn't match any real password
    // Using m=19456,t=2,p=1 to match the parameters in create_argon2()
    "$argon2id$v=19$m=19456,t=2,p=1$dHlwaWNhbHNhbHQ$GvIkHHfKCGGlPnBbFx2v7g+oL1wP0TG1+Ck0gT7gMxA"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let password = "test_password_123!";
        let hash = hash_password(password).expect("Failed to hash password");

        // Password should match
        assert!(verify_password(password, &hash).expect("Verification error"));

        // Wrong password should not match
        assert!(!verify_password("wrong_password", &hash).expect("Verification error"));
    }
}

#[cfg(test)]
mod async_tests {
    use super::*;

    #[tokio::test]
    async fn test_async_hash_and_verify_round_trip() {
        let hash = hash_password_async("Round-trip-1".to_string()).await.expect("hash");
        assert!(verify_password_async("Round-trip-1".to_string(), hash.clone()).await.expect("verify"));
        assert!(!verify_password_async("wrong".to_string(), hash).await.expect("verify"));
    }
}
