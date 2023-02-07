-- Add down migration script here
ALTER TABLE populist_user
DROP COLUMN last_login_at;