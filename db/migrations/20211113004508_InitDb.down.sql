-- Add down migration script here
DROP EXTENSION fuzzystrmatch;
DROP EXTENSION pgcrypto;

DROP TYPE vote_status;
DROP TYPE state;

DROP TABLE politician;
DROP TABLE organization;
DROP TABLE election;
DROP TABLE election;
DROP TABLE legislation;
DROP TABLE bill;
DROP TABLE ballot_measure;
DROP TABLE politician_endorsements;
DROP TABLE politician_legislation;
DROP TABLE organization_legislation;