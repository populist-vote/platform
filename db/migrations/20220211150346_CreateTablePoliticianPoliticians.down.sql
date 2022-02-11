-- Add down migration script here
ALTER TABLE IF EXISTS politician_organization_endorsements
RENAME TO politician_endorsements;

DROP TABLE politician_politician_endorsements;