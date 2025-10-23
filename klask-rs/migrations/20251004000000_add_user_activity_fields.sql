-- Add last_login and last_activity columns to users table
ALTER TABLE users
ADD COLUMN last_login TIMESTAMP WITH TIME ZONE,
ADD COLUMN last_activity TIMESTAMP WITH TIME ZONE;
