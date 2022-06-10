-- Add down migration script here
ALTER TABLE politician
ADD COLUMN nickname TEXT,
ADD COLUMN ballot_name TEXT,
DROP COLUMN race_wins,
DROP COLUMN race_losses;