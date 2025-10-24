-- Increase avatar_url column size to support base64 data URIs
-- Base64 encoded images can be quite large (typical avatar ~100KB = ~133KB base64)
ALTER TABLE users
ALTER COLUMN avatar_url SET DATA TYPE TEXT;

-- Drop the old constraint and add a new one with larger limit
ALTER TABLE users
DROP CONSTRAINT check_avatar_url_length,
ADD CONSTRAINT check_avatar_url_length CHECK (length(avatar_url) <= 1000000);
