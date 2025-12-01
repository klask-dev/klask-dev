-- Add last_crawl_error column to repositories table
-- This will store the error message from the last failed crawl

ALTER TABLE repositories
ADD COLUMN last_crawl_error TEXT;

-- Add comment to document the field
COMMENT ON COLUMN repositories.last_crawl_error IS 'Error message from the last failed crawl, cleared on successful crawl';
