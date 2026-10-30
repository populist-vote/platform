BEGIN;

ALTER TABLE ballot_measure
DROP COLUMN subdistrict,
DROP COLUMN district,
DROP COLUMN district_type;

COMMIT;
