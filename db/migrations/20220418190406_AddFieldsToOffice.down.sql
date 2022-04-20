-- Add down migration script here
ALTER TABLE office
DROP COLUMN IF EXISTS election_scope,
DROP COLUMN IF EXISTS district_type,
DROP COLUMN IF EXISTS chamber;

DROP TYPE IF EXISTS election_scope;
DROP TYPE IF EXISTS district_type;
DROP TYPE IF EXISTS chamber;