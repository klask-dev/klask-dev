-- Update crawl_state documentation to include 'stopped' state
COMMENT ON COLUMN repositories.crawl_state IS 'Track current crawl state: idle, in_progress, failed, stopped';
