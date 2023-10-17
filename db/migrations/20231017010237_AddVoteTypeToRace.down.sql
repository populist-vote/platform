-- Add down migration script here
ALTER TABLE race DROP COLUMN vote_type;
DROP TYPE vote_type;
