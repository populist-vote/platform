-- Add down migration script here
ALTER TABLE populist_user
DROP COLUMN organization_id;