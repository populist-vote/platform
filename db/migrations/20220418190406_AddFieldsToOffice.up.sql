-- Add up migration script here
CREATE TYPE election_scope AS ENUM ('national', 'state', 'county', 'city', 'district');
CREATE TYPE district_type AS ENUM ('us_congressional', 'state_senate', 'state_house', 'school');
CREATE TYPE chamber AS ENUM ('house', 'senate');

ALTER TABLE office
ADD COLUMN election_scope election_scope NOT NULL DEFAULT 'national',
ADD COLUMN district_type district_type,
ADD COLUMN chamber chamber;
