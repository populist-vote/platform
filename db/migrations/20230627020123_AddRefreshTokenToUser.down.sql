-- Add down migration script here
ALTER TABLE populist_user 
DROP COLUMN IF EXISTS refresh_token,
DROP COLUMN IF EXISTS invited_at;
