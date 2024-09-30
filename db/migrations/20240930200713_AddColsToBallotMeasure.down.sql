-- Add down migration script here
ALTER TABLE ballot_measure
DROP COLUMN election_scope,
DROP COLUMN county,
DROP COLUMN municipality,
DROP COLUMN school_district;

ALTER TABLE ballot_measure
RENAME COLUMN state TO ballot_state;
