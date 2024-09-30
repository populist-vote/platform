-- Add up migration script here

ALTER TABLE ballot_measure
ADD COLUMN election_scope election_scope,
ADD COLUMN county text,
ADD COLUMN municipality text,
ADD COLUMN school_district text;

ALTER TABLE ballot_measure
RENAME COLUMN ballot_state TO state;
