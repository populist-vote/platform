-- Add down migration script here
ALTER TABLE race_candidates
DROP COLUMN ref_key;
