-- Add down migration script here
ALTER TABLE race
DROP COLUMN is_special_election,
DROP COLUMN num_elect;