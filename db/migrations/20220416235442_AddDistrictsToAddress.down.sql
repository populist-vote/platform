-- Add down migration script here
ALTER TABLE address 
DROP COLUMN congressional_district,
DROP COLUMN state_senate_district,
DROP COLUMN state_house_district;