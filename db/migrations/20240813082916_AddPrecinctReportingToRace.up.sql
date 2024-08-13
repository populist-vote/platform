-- Add up migration script here
ALTER TABLE race
ADD COLUMN num_precincts_reporting INTEGER,
ADD COLUMN total_precincts INTEGER
