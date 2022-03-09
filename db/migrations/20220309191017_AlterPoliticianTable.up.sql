-- Add up migration script here
ALTER TABLE politician
ALTER COLUMN home_state
DROP NOT NULL;

ALTER TABLE office
ALTER COLUMN incumbent_id
DROP NOT NULL;