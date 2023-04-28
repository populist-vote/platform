-- Add down migration script here
ALTER TABLE bill
DROP COLUMN is_locked;