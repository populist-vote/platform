-- Add up migration script here
ALTER TABLE politician_politician_endorsements
ADD COLUMN start_date DATE,
ADD COLUMN end_date DATE;

ALTER TABLE politician_organization_endorsements
ADD COLUMN start_date DATE,
ADD COLUMN end_date DATE;
