-- Add down migration script here
-- Add up migration script here
ALTER TABLE candidate_guide
DROP COLUMN submissions_open_at,
DROP COLUMN submissions_close_at;
