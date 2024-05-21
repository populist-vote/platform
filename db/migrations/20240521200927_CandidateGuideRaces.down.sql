-- Add down migration script here
ALTER TABLE candidate_guide
ADD COLUMN race_id uuid REFERENCES race (id) ON DELETE CASCADE;

DROP TABLE candidate_guide_races;
