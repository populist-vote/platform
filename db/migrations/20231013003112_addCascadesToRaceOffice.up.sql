-- Add up migration script here
ALTER TABLE race_candidates
DROP CONSTRAINT IF EXISTS fk_race;

ALTER TABLE race_candidates 
ADD CONSTRAINT fk_race
FOREIGN KEY (race_id) REFERENCES race(id)
ON DELETE CASCADE;

ALTER TABLE race
DROP CONSTRAINT IF EXISTS fk_office;

ALTER TABLE race
ADD CONSTRAINT fk_office
FOREIGN KEY (office_id) REFERENCES office(id)
ON DELETE CASCADE;