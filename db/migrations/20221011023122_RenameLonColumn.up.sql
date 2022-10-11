-- Add up migration script here
-- Naming convention is lat/lon
ALTER TABLE address RENAME COLUMN long TO lon;
