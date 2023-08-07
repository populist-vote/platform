-- Add up migration script here
ALTER TABLE election ADD UNIQUE (slug);