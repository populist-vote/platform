-- Add down migration script here
DROP INDEX IF EXISTS idx_organization_politician_id;
ALTER TABLE organization
DROP COLUMN politician_id;

DROP TABLE invite_token;
