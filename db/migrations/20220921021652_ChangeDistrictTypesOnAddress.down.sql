-- Add down migration script here
ALTER TABLE address 
ALTER COLUMN congressional_district TYPE INTEGER USING congressional_district::integer,
ALTER COLUMN state_senate_district TYPE INTEGER USING state_senate_district::integer,
ALTER COLUMN state_house_district TYPE INTEGER USING state_house_district::integer;