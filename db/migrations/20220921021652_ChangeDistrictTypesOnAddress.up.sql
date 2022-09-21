-- Add up migration script here
ALTER TABLE address 
ALTER COLUMN congressional_district TYPE TEXT,
ALTER COLUMN state_senate_district TYPE TEXT,
ALTER COLUMN state_house_district TYPE TEXT;