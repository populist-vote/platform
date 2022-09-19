-- Add down migration script here
ALTER TABLE office 
DROP COLUMN county,
DROP COLUMN hospital_district,
DROP COLUMN school_district;