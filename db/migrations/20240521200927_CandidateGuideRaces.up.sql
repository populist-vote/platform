-- Add up migration script here
ALTER TABLE candidate_guide
DROP COLUMN race_id;

CREATE TABLE candidate_guide_races (
    candidate_guide_id uuid REFERENCES candidate_guide (id) ON DELETE CASCADE,
    race_id uuid REFERENCES race (id) ON DELETE CASCADE
);
