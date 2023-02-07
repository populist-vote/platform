-- Add up migration script here
ALTER TABLE populist_user  
ADD COLUMN last_login_at TIMESTAMPTZ;