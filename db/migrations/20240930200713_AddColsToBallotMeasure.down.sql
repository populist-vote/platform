-- Add down migration script here
ALTER TABLE ballot_measure
DROP COLUMN election_scope,
DROP COLUMN county,
DROP COLUMN municipality,
DROP COLUMN school_district,
DROP COLUMN yes_votes,
DROP COLUMN no_votes,
DROP COLUMN num_precincts_reporting,
DROP COLUMN total_precincts;

ALTER TABLE ballot_measure
RENAME COLUMN state TO ballot_state;
