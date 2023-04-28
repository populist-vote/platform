-- Add up migration script here
ALTER TABLE bill 
ADD COLUMN is_locked BOOLEAN NOT NULL DEFAULT FALSE;
