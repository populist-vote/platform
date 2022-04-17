-- Add up migration script here
ALTER TABLE address 
ADD COLUMN congressional_district INTEGER,
ADD COLUMN state_senate_district INTEGER,
ADD COLUMN state_house_district INTEGER;