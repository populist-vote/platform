-- Add up migration script here
ALTER TABLE election DROP COLUMN state;

ALTER TABLE RACE
ADD COLUMN winner_id uuid,
DROP COLUMN election_date;