-- Add down migration script here
ALTER TABLE address RENAME COLUMN lon TO long;
