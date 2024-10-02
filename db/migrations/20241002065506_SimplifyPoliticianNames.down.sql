-- Add down migration script here
ALTER TABLE politician ALTER COLUMN first_name SET NOT NULL;
ALTER TABLE politician ALTER COLUMN last_name SET NOT NULL;
ALTER TABLE politician DROP COLUMN display_name;
