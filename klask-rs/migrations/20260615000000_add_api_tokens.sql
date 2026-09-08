-- Create api_tokens table for personal API token management
-- Tokens follow format: klask_pat_<32 random chars>
-- Tokens are hashed using Argon2 before storage (similar to passwords)

CREATE TABLE api_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL UNIQUE,  -- Argon2 hash of the full token
    token_prefix VARCHAR(12) NOT NULL,         -- "klask_pat_" + first 2 chars for display
    name VARCHAR(255) NOT NULL,                -- User-friendly name (e.g., "GitHub Actions CI")
    scope VARCHAR(50) NOT NULL DEFAULT 'read-only',  -- read-only, read-write, etc.
    active BOOLEAN NOT NULL DEFAULT TRUE,      -- Soft delete (false = revoked)
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMP WITH TIME ZONE,     -- For audit trail
    expires_at TIMESTAMP WITH TIME ZONE,       -- Optional expiration

    -- Ensure non-empty fields
    CHECK (length(trim(token_hash)) > 0),
    CHECK (length(trim(token_prefix)) > 0),
    CHECK (length(trim(name)) > 0),
    CHECK (length(trim(scope)) > 0)
);

-- Indexes for common queries
CREATE INDEX idx_api_tokens_user_id ON api_tokens(user_id);
CREATE INDEX idx_api_tokens_user_id_active ON api_tokens(user_id, active);
CREATE INDEX idx_api_tokens_user_id_created_at ON api_tokens(user_id, created_at DESC);
CREATE INDEX idx_api_tokens_token_hash ON api_tokens(token_hash);  -- For quick lookup during auth

-- Comment documenting the token format and security model
COMMENT ON TABLE api_tokens IS 'Personal API tokens for programmatic access. Tokens are hashed using Argon2 before storage.';
COMMENT ON COLUMN api_tokens.token_hash IS 'Argon2 hash of the full token (klask_pat_XXXXX). Never expose plaintext after creation.';
COMMENT ON COLUMN api_tokens.token_prefix IS 'Display-safe prefix of token for UI (e.g., "klask_pat_ab" from "klask_pat_ab12...").';
COMMENT ON COLUMN api_tokens.scope IS 'Token scope/permissions. Currently: read-only. Future: read-write, write-only.';
COMMENT ON COLUMN api_tokens.active IS 'Soft delete flag. false = token has been revoked.';
