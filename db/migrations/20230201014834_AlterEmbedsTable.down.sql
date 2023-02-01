-- Add down migration script here
ALTER TABLE embed
DROP COLUMN created_by,
DROP COLUMN updated_by;
DROP TRIGGER set_updated_at ON embed;