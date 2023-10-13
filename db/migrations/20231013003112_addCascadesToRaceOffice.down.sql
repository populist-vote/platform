-- Add down migration script here
ALTER TABLE race_candidates
DROP CONSTRAINT IF EXISTS fk_race_id;

ALTER TABLE race
DROP CONSTRAINT IF EXISTS fk_office_id;

