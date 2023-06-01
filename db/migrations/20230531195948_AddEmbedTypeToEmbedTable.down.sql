-- Add down migration script here
ALTER TABLE embed DROP COLUMN embed_type;
DROP TYPE IF EXISTS embed_type;
