-- Add down migration script here
ALTER TABLE politician
DROP CONSTRAINT fk_author_politician;

ALTER TABLE organization
DROP CONSTRAINT fk_author_organization;


DROP TABLE bill_arguments;
DROP TABLE ballot_measure_arguments;
DROP TABLE argument;
DROP TABLE author;
DROP TYPE argument_position;
DROP TYPE author_type;