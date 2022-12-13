-- Add down migration script here
CREATE TYPE legislation_status AS ENUM ('introduced', 'passed_house', 'passed_senate', 'failed_house', 'failed_senate', 'resolving_differences', 'sent_to_executive', 'became_law', 'failed', 'vetoed', 'unknown');
ALTER TABLE legislation ADD COLUMN legislation_status legislation_status NOT NULL DEFAULT 'unknown';
ALTER TABLE bill DROP COLUMN status;
ALTER TABLE ballot_measure DROP COLUMN status;
DROP TYPE bill_status;
DROP TYPE ballot_measure_status;