-- Add down migration script here
ALTER TABLE race
ADD COLUMN office_position TEXT NOT NULL;