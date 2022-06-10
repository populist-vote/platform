-- Add up migration script here

ALTER TABLE politician
DROP COLUMN nickname,
DROP COLUMN ballot_name,
ADD COLUMN race_wins INT DEFAULT(0),
ADD COLUMN race_losses INT DEFAULT(0);