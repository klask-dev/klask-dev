-- Add GitHub support to Klask

-- Note: repository_type is a VARCHAR(20), not a PostgreSQL enum
-- So we don't need to modify any type, just add the new columns

-- Add GitHub-specific fields to repositories table
ALTER TABLE repositories
    ADD COLUMN IF NOT EXISTS github_namespace TEXT,
    ADD COLUMN IF NOT EXISTS github_excluded_repositories TEXT,
    ADD COLUMN IF NOT EXISTS github_excluded_patterns TEXT;

-- Add comments to explain the new columns
COMMENT ON COLUMN repositories.github_namespace IS 'GitHub organization or user to filter repositories (e.g., "my-org")';
COMMENT ON COLUMN repositories.github_excluded_repositories IS 'Comma-separated list of GitHub repositories to exclude (e.g., "owner/repo1,owner/repo2")';
COMMENT ON COLUMN repositories.github_excluded_patterns IS 'Comma-separated list of wildcard patterns to exclude GitHub repositories (e.g., "*-archive,test/*")';
