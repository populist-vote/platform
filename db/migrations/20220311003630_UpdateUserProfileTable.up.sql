-- Add up migration script here
ALTER TABLE user_profile
ADD COLUMN confirmed_at timestamptz,
ADD COLUMN updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
ADD COLUMN party political_party;

ALTER TABLE populist_user
ADD COLUMN reset_token TEXT,
ADD COLUMN reset_token_expires_at timestamptz,
ADD COLUMN confirmation_token TEXT;