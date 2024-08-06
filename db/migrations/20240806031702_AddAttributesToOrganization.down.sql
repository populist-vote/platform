-- Add down migration script here
ALTER TABLE organization
DROP COLUMN attributes;
