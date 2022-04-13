-- Add down migration script here
ALTER TABLE election ADD COLUMN state state;

ALTER TABLE race
DROP COLUMN winner_id,
ADD COLUMN election_date DATE; 