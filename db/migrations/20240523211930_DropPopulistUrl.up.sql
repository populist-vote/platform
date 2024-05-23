-- Add up migration script here
ALTER TABLE embed DROP COLUMN IF EXISTS populist_url;

ALTER TABLE candidate_guide DROP COLUMN IF EXISTS populist_url;
