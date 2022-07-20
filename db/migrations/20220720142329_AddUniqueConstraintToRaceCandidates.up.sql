-- Add up migration script here
ALTER TABLE race_candidates ADD CONSTRAINT unique_race_candidate UNIQUE (race_id, candidate_id);