-- Add down migration script here
ALTER TABLE politician
DROP COLUMN ref_key;

ALTER TABLE office
DROP COLUMN state_id,
DROP COLUMN ref_key;