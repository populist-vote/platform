-- Add up migration script here
CREATE TYPE district_type_new AS ENUM ('us_congressional', 'state_senate', 'state_house', 'school', 'city', 'county');
ALTER TABLE office ALTER COLUMN district_type TYPE district_type_new USING district_type::text::district_type_new;
DROP TYPE district_type;
ALTER TYPE district_type_new RENAME TO district_type;