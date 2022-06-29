-- Add up migration script here
ALTER TABLE race_candidates
ADD COLUMN votes INTEGER;

ALTER TABLE race
ADD COLUMN total_votes INTEGER;