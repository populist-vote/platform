-- Add up migration script here

ALTER TABLE race_candidates
ADD COLUMN ref_key TEXT;
