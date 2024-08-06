-- Add up migration script here
ALTER TABLE organization
ADD COLUMN attributes JSONB DEFAULT '{}'::JSONB;
