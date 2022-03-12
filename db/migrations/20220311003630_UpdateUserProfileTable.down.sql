-- Add down migration script here
ALTER TABLE user_profile
DROP COLUMN confirmed_at,
DROP COLUMN updated_at,
DROP COLUMN party;

ALTER TABLE populist_user
DROP COLUMN reset_token,
DROP COLUMN reset_token_expires_at,
DROP COLUMN confirmation_token;