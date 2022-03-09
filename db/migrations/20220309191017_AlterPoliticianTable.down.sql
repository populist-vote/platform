-- Add down migration script here
ALTER TABLE politician
ALTER COLUMN home_state
SET NOT NULL;

ALTER TABLE office
ALTER COLUMN incumbent_id
SET NOT NULL;