-- Add up migration script here
ALTER TABLE ballot_measure
ALTER COLUMN measure_type DROP NOT NULL,
ALTER COLUMN definitions DROP NOT NULL;

ALTER TABLE ballot_measure
ADD COLUMN county_fips TEXT,
ADD COLUMN municipality_fips TEXT;
