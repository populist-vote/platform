-- Add down migration script here
ALTER TABLE address 
DROP COLUMN IF EXISTS county,
DROP COLUMN IF EXISTS congressional_district,
DROP COLUMN IF EXISTS state_senate_district,
DROP COLUMN IF EXISTS state_house_district;