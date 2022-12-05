-- Add down migration script here
CREATE TYPE chamber_new AS ENUM ('house', 'senate');

ALTER TABLE office
ALTER COLUMN chamber TYPE chamber_new USING chamber::text::chamber_new;

ALTER TABLE bill
ALTER COLUMN chamber TYPE chamber_new USING chamber::text::chamber_new;

DROP TYPE chamber;
ALTER TYPE chamber_new RENAME TO chamber;

ALTER TABLE bill
DROP COLUMN political_scope,
DROP COLUMN bill_type,
DROP COLUMN chamber,
DROP COLUMN attributes;

DROP TABLE bill_public_votes;
DROP TABLE ballot_measure_public_votes;