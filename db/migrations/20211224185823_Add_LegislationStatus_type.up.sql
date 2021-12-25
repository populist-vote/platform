-- Add down migration script here
CREATE TYPE legislation_status AS ENUM ('introduced', 'passed_house', 'passed_senate', 'failed_house', 'failed_senate', 'resolving_differences', 'sent_to_executive', 'became_law', 'vetoed', 'unknown');

ALTER TABLE legislation
DROP COLUMN vote_status;

ALTER TABLE legislation
ADD COLUMN legislation_status legislation_status NOT NULL;

DROP TYPE vote_status;