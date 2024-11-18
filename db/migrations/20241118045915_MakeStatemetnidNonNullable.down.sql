-- Add down migration script here
ALTER TABLE statement_vote ALTER COLUMN statement_id DROP NOT NULL;

ALTER TABLE statement_vote DROP COLUMN updated_at;

DROP TRIGGER IF EXISTS set_updated_at ON statement_vote;
