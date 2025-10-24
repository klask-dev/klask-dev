-- Add user profile fields to users table
ALTER TABLE users
ADD COLUMN avatar_url VARCHAR(500),
ADD COLUMN bio TEXT,
ADD COLUMN full_name VARCHAR(255),
ADD COLUMN phone VARCHAR(20),
ADD COLUMN timezone VARCHAR(50) DEFAULT 'UTC',
ADD COLUMN preferences JSONB DEFAULT '{}',
ADD COLUMN login_count INTEGER DEFAULT 0;

-- Add CHECK constraints for field lengths and valid values
ALTER TABLE users
ADD CONSTRAINT check_avatar_url_length CHECK (length(avatar_url) <= 500),
ADD CONSTRAINT check_full_name_length CHECK (length(full_name) <= 255),
ADD CONSTRAINT check_phone_length CHECK (length(phone) <= 20),
ADD CONSTRAINT check_bio_length CHECK (length(bio) <= 2000),
ADD CONSTRAINT check_timezone_length CHECK (length(timezone) <= 50),
ADD CONSTRAINT check_login_count_positive CHECK (login_count >= 0);

-- Create indexes for better query performance
CREATE INDEX idx_users_timezone ON users(timezone);
CREATE INDEX idx_users_login_count ON users(login_count);
CREATE INDEX idx_users_updated_at ON users(updated_at);

-- Update trigger to handle new timestamp for last_activity updates
-- The existing trigger already handles updated_at automatically
