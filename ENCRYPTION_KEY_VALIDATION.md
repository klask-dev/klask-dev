# ENCRYPTION_KEY Validation at Startup

## Summary

Added validation of the `ENCRYPTION_KEY` environment variable at application startup to catch configuration issues early, before the server starts processing requests.

## Problem Solved

Previously, if the `ENCRYPTION_KEY` was:
- Missing → Server used a hardcoded default value (security issue)
- Empty → No error until tokens needed decryption
- Invalid/changed → Silently failed to decrypt tokens when needed

Now, these issues are caught immediately at startup with clear error messages.

## Implementation

### Changes Made

**File: `klask-rs/src/main.rs`**

#### 1. Added Database-Backed Validation Function (before main)

```rust
/// Validate that the encryption service can decrypt existing tokens in the database
/// This ensures the ENCRYPTION_KEY hasn't changed since the tokens were encrypted
/// If there are no tokens, performs a basic roundtrip test instead
async fn validate_encryption_service(service: &EncryptionService, database: &Database) -> Result<()> {
    // Try to find a repository with an encrypted access token
    let result = sqlx::query(
        "SELECT id, access_token FROM repositories WHERE access_token IS NOT NULL LIMIT 1"
    )
    .fetch_optional(database.pool())
    .await;

    match result {
        Ok(Some(row)) => {
            // Found an encrypted token in the database, try to decrypt it
            info!("Found encrypted token in database, validating ENCRYPTION_KEY...");

            let encrypted_token = row.access_token
                .ok_or_else(|| anyhow::anyhow!("Token field is null"))?;

            match service.decrypt(&encrypted_token) {
                Ok(decrypted) => {
                    if decrypted.is_empty() {
                        return Err(anyhow::anyhow!("ENCRYPTION_KEY validation failed: decrypted token is empty"));
                    }
                    info!("Successfully decrypted existing token from database - ENCRYPTION_KEY is valid");
                    Ok(())
                }
                Err(e) => {
                    error!("ENCRYPTION_KEY validation FAILED: Cannot decrypt existing tokens in database");
                    error!("This likely means the ENCRYPTION_KEY has changed since the tokens were encrypted");
                    error!("Please restore the correct ENCRYPTION_KEY or clear the encrypted tokens from the database");
                    Err(anyhow::anyhow!("Failed to decrypt existing token: {}. ENCRYPTION_KEY may have changed.", e))
                }
            }
        }
        Ok(None) => {
            // No encrypted tokens in database, perform a basic roundtrip test
            info!("No existing tokens in database, performing basic encryption validation...");

            const TEST_DATA: &str = "encryption_key_validation_test";

            let encrypted = service.encrypt(TEST_DATA)
                .map_err(|e| anyhow::anyhow!("Encryption validation failed: {}", e))?;

            let decrypted = service.decrypt(&encrypted)
                .map_err(|e| anyhow::anyhow!("Decryption validation failed: {}", e))?;

            if decrypted != TEST_DATA {
                return Err(anyhow::anyhow!("Encryption/decryption roundtrip failed: decrypted data does not match original"));
            }

            info!("Basic encryption validation passed");
            Ok(())
        }
        Err(e) => {
            error!("Failed to query database for token validation: {}", e);
            Err(anyhow::anyhow!("Database query failed during ENCRYPTION_KEY validation: {}", e))
        }
    }
}
```

**Purpose:**
- Tries to decrypt an existing token from the database (real-world data)
- Detects if ENCRYPTION_KEY has changed since tokens were created
- Falls back to basic roundtrip test if no tokens exist yet
- Fails startup if the key is incompatible with stored data
- Provides clear error messages to help operators recover

#### 2. Enhanced Environment Variable Validation

```rust
let encryption_key = match std::env::var("ENCRYPTION_KEY") {
    Ok(key) => {
        // Check 1: Key is not empty
        if key.is_empty() {
            error!("ENCRYPTION_KEY environment variable is empty. Please provide a non-empty encryption key.");
            return Err(anyhow::anyhow!("ENCRYPTION_KEY is empty"));
        }

        // Check 2: Key meets minimum length
        if key.len() < 16 {
            error!("ENCRYPTION_KEY must be at least 16 characters long. Current length: {}", key.len());
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
```

**Validations:**
1. **Required** - Fails if `ENCRYPTION_KEY` is not set (no default fallback)
2. **Non-empty** - Fails if the variable is set but empty
3. **Minimum length** - Requires at least 16 characters
4. **Functional** - Tests encryption/decryption roundtrip

#### 3. Initialization with Validation

```rust
let encryption_service = match EncryptionService::new(&encryption_key) {
    Ok(service) => {
        // Validate encryption service by testing encrypt/decrypt roundtrip
        match validate_encryption_service(&service) {
            Ok(_) => {
                info!("Encryption service initialized and validated successfully");
                Arc::new(service)
            }
            Err(e) => {
                error!("Encryption service validation failed: {}", e);
                return Err(e);
            }
        }
    }
    Err(e) => {
        error!("Failed to initialize encryption service: {}", e);
        return Err(e);
    }
};
```

**Flow:**
1. Create cipher from ENCRYPTION_KEY
2. Test that encryption works
3. Test that decryption works
4. Verify roundtrip data integrity
5. Log success or fail startup

## Behavior Changes

### Before

```bash
# Missing ENCRYPTION_KEY
$ helm install klask ./charts/klask
# Server starts with hardcoded default key (SECURITY ISSUE!)

# Empty ENCRYPTION_KEY
$ ENCRYPTION_KEY="" helm install klask ./charts/klask
# Server starts, fails later when trying to decrypt tokens
# Error: "Decryption failed" in logs (unclear)

# Wrong ENCRYPTION_KEY (with existing tokens in database)
$ ENCRYPTION_KEY="new-key" helm install klask ./charts/klask
# Server starts, but can't access repos that use tokens
# Error discovered only when accessing specific repositories
# No indication at startup that something is wrong
```

### After

```bash
# Missing ENCRYPTION_KEY
$ helm install klask ./charts/klask
# ❌ STARTUP FAILS immediately
# Error: ENCRYPTION_KEY environment variable is not set. This is required for secure token storage.
# Set ENCRYPTION_KEY to a random string of at least 16 characters.
# Generate one with: openssl rand -hex 32

# Empty ENCRYPTION_KEY
$ ENCRYPTION_KEY="" helm install klask ./charts/klask
# ❌ STARTUP FAILS immediately
# Error: ENCRYPTION_KEY environment variable is empty. Please provide a non-empty encryption key.

# Too-short ENCRYPTION_KEY
$ ENCRYPTION_KEY="short" helm install klask ./charts/klask
# ❌ STARTUP FAILS immediately
# Error: ENCRYPTION_KEY must be at least 16 characters long. Current length: 5

# Wrong ENCRYPTION_KEY (with existing tokens in database)
$ ENCRYPTION_KEY="wrong-key-12345" helm install klask ./charts/klask
# ❌ STARTUP FAILS immediately with CLEAR ERROR
# Logs:
#   Found encrypted token in database, validating ENCRYPTION_KEY...
#   ENCRYPTION_KEY validation FAILED: Cannot decrypt existing tokens in database
#   This likely means the ENCRYPTION_KEY has changed since the tokens were encrypted
#   Please restore the correct ENCRYPTION_KEY or clear the encrypted tokens from the database

# Correct ENCRYPTION_KEY with existing tokens
$ ENCRYPTION_KEY="correct-key-32-chars-minimum" helm install klask ./charts/klask
# ✅ SERVER STARTS SUCCESSFULLY
# Logs: Successfully decrypted existing token from database - ENCRYPTION_KEY is valid

# Fresh deployment (no tokens yet)
$ ENCRYPTION_KEY="new-key-32-chars-minimum" helm install klask ./charts/klask
# ✅ SERVER STARTS SUCCESSFULLY
# Logs: No existing tokens in database, performing basic encryption validation...
#       Basic encryption validation passed
```

## Deployment Impact

### For Kubernetes/Helm

The Helm chart `backend.auth.encryptionKey` must be provided:

```yaml
# values.yaml or custom-values.yaml
backend:
  auth:
    encryptionKey: "your-random-32-char-key-here"
    jwtSecret: "your-random-32-char-key-here"
```

Or via environment:
```bash
helm install klask ./charts/klask \
  --set backend.auth.encryptionKey="$(openssl rand -hex 32)"
```

**Important:** The chart will now **fail to deploy** if `ENCRYPTION_KEY` is not properly configured, preventing silent failures.

### For Docker/Direct Deployment

```bash
# Must set ENCRYPTION_KEY before starting
export ENCRYPTION_KEY="$(openssl rand -hex 32)"
docker run -e ENCRYPTION_KEY="$ENCRYPTION_KEY" klask-backend:latest

# Or in docker-compose.yml
services:
  backend:
    environment:
      ENCRYPTION_KEY: ${ENCRYPTION_KEY:-}  # Fails if not set
```

## Security Improvements

✅ **No default fallback** - Prevents accidental use of insecure defaults
✅ **Minimum length enforcement** - Ensures adequate key entropy
✅ **Functional validation** - Catches misconfigured keys before production impact
✅ **Clear error messages** - Helps operators fix configuration issues
✅ **Fail-fast approach** - Better than silent failures

## Error Messages Guide

### Error: ENCRYPTION_KEY environment variable is not set

**Cause:** The environment variable is not defined
**Fix:** Set ENCRYPTION_KEY before starting the server
```bash
export ENCRYPTION_KEY="$(openssl rand -hex 32)"
```

### Error: ENCRYPTION_KEY environment variable is empty

**Cause:** ENCRYPTION_KEY is set to empty string
**Fix:** Provide a non-empty value
```bash
export ENCRYPTION_KEY="$(openssl rand -hex 32)"
```

### Error: ENCRYPTION_KEY is too short

**Cause:** Key is less than 16 characters
**Fix:** Generate a longer key
```bash
export ENCRYPTION_KEY="$(openssl rand -hex 32)"
# Or use your own 16+ character string
```

### Error: Encryption service validation failed

**Cause:** The encryption service can't perform roundtrip encryption/decryption
**Possible reasons:**
- Key corruption
- Incompatible key format
- Cipher initialization failure

**Fix:** Verify the ENCRYPTION_KEY is correct and not corrupted:
```bash
# Generate a new key
export ENCRYPTION_KEY="$(openssl rand -hex 32)"
```

## Testing the Implementation

### Manual Test

```bash
# Start with valid key
export ENCRYPTION_KEY="$(openssl rand -hex 32)"
cargo run
# Should log: "Encryption service initialized and validated successfully"

# Test with missing key
unset ENCRYPTION_KEY
cargo run
# Should fail with: "ENCRYPTION_KEY environment variable is not set"

# Test with empty key
export ENCRYPTION_KEY=""
cargo run
# Should fail with: "ENCRYPTION_KEY environment variable is empty"

# Test with short key
export ENCRYPTION_KEY="short"
cargo run
# Should fail with: "ENCRYPTION_KEY is too short"
```

### Kubernetes Test

```bash
# Deploy without ENCRYPTION_KEY
kubectl set env deployment/klask-backend ENCRYPTION_KEY- --overwrite
kubectl rollout restart deployment/klask-backend

# Pod will fail to start
kubectl logs pod/klask-backend-xxx
# Error: ENCRYPTION_KEY environment variable is not set

# Fix by setting proper value
kubectl set env deployment/klask-backend \
  ENCRYPTION_KEY="$(openssl rand -hex 32)" \
  --overwrite
kubectl rollout restart deployment/klask-backend

# Pod should start successfully
```

## Compatibility

✅ **Backward compatible** with existing valid deployments
✅ **More strict** with invalid configurations
❌ **Breaks** if ENCRYPTION_KEY was not being set (now required)

### Migration for Existing Deployments

If you have existing deployments without ENCRYPTION_KEY:

1. Generate a secure key:
   ```bash
   ENCRYPTION_KEY="$(openssl rand -hex 32)"
   ```

2. Update deployment:
   ```bash
   # For Helm
   helm upgrade klask ./charts/klask \
     --set backend.auth.encryptionKey="$ENCRYPTION_KEY"

   # For Kubernetes directly
   kubectl set env deployment/klask-backend \
     ENCRYPTION_KEY="$ENCRYPTION_KEY"
   ```

3. Restart pods:
   ```bash
   kubectl rollout restart deployment/klask-backend
   ```

## Future Enhancements

Possible future improvements:
- Support for external key management (HashiCorp Vault, AWS KMS)
- Key rotation support with multiple keys
- Warning if key entropy is low
- Audit logging of key-related operations
- Support for hardware security modules (HSM)

## Summary

This change improves operational reliability by:
1. **Failing fast** - Catches configuration issues immediately
2. **Clear feedback** - Helpful error messages guide operators
3. **Security** - Eliminates insecure defaults
4. **Testing** - Validates the encryption system before processing real data
