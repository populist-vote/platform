-- Add up migration script here
ALTER TABLE race ADD COLUMN canceled_at TIMESTAMP WITH TIME ZONE;
