-- Add up migration script here
CREATE TYPE vote_status AS ENUM ('introduced', 'passed', 'signed', 'vetoed', 'unknown');

ALTER TABLE legislation
DROP COLUMN legislation_status;

ALTER TABLE legislation
ADD COLUMN vote_status vote_status NOT NULL;

DROP TYPE legislation_status;

