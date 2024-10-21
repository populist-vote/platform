-- Add down migration script here
UPDATE ballot_measure
SET
    measure_type = 'null',
    definitions = 'null';

ALTER TABLE ballot_measure
ALTER COLUMN measure_type SET NOT NULL,
ALTER COLUMN definitions SET NOT NULL;

ALTER TABLE ballot_measure
DROP COLUMN county_fips,
DROP COLUMN municipality_fips;
