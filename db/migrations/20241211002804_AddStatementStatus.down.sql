-- Add down migration script here
DROP INDEX IF EXISTS idx_statement_moderation_status;

ALTER TABLE statement
DROP COLUMN moderation_status;

DROP TYPE IF EXISTS statement_moderation_status;
