-- Add down migration script here
ALTER TABLE politician_politician_endorsements
DROP COLUMN election_id;

ALTER TABLE politician_organization_endorsements
DROP COLUMN election_id;
