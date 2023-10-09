-- Add up migration script here
ALTER TABLE politician
ADD COLUMN ref_key TEXT UNIQUE;

ALTER TABLE office
ADD COLUMN state_id TEXT,
ADD COLUMN ref_key TEXT UNIQUE;
