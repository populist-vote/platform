-- Add down migration script here

UPDATE office SET district_type = NULL WHERE district_type = 'transportation';

ALTER TYPE district_type RENAME TO district_type_old;
CREATE TYPE district_type AS ENUM('us_congressional', 'state_senate', 'state_house', 'school', 'city', 'county', 'hospital', 'judicial', 'soil_and_water');
ALTER TABLE office ALTER COLUMN district_type TYPE district_type USING district_type::text::district_type;
DROP TYPE district_type_old;