BEGIN;

ALTER TABLE ballot_measure
ADD COLUMN district_type district_type,
ADD COLUMN district TEXT,
ADD COLUMN subdistrict TEXT;

COMMIT;
