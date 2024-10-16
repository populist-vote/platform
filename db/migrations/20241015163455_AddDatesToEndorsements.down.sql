-- Add down migration script here
ALTER TABLE politician_politician_endorsements
DROP COLUMN start_date,
DROP COLUMN end_date;

ALTER TABLE politician_organization_endorsements
DROP COLUMN start_date,
DROP COLUMN end_date;
