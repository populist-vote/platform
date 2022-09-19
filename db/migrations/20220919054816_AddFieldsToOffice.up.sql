-- Add up migration script here
ALTER TABLE office 
ADD COLUMN county TEXT,
ADD COLUMN hospital_district TEXT,
ADD COLUMN school_district TEXT;