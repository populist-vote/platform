-- Add down migration script here
ALTER TABLE race_candidates
DROP COLUMN votes;

ALTER TABLE race
DROP COLUMN total_votes;