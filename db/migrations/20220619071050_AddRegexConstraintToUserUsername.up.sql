-- Add up migration script here
ALTER TABLE populist_user ADD CONSTRAINT check_valid_chars CHECK ( username ~ '^[a-zA-Z0-9_\-]+$' );