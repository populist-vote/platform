-- Add down migration script here
ALTER TABLE candidate_guide_races
DROP COLUMN were_candidates_emailed,
DROP COLUMN IF EXISTS created_at,
DROP COLUMN IF EXISTS updated_at;

DROP TRIGGER IF EXISTS set_updated_at
ON candidate_guide_races;
