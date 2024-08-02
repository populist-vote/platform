-- Add down migration script here
DROP INDEX IF EXISTS idx_organization_id;
ALTER TABLE politician
DROP COLUMN organization_id;

DROP TABLE invite_token;
