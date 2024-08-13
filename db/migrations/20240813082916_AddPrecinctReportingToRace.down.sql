-- Add down migration script here
ALTER TABLE race
DROP COLUMN num_precincts_reporting,
DROP COLUMN total_precincts;
