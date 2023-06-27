-- Add up migration script here
ALTER TABLE populist_user 
ADD COLUMN refresh_token TEXT,
ADD COLUMN invited_at TIMESTAMPTZ;
