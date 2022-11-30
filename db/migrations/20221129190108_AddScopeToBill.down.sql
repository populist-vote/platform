-- Add down migration script here
ALTER TABLE bill
DROP COLUMN political_scope,
DROP COLUMN bill_type,
DROP COLUMN attributes;

DROP TABLE bill_public_votes;
DROP TABLE ballot_measure_public_votes;