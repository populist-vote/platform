-- Add down migration script here
ALTER TABLE embed ADD COLUMN IF NOT EXISTS populist_url TEXT;
ALTER TABLE candidate_guide ADD COLUMN IF NOT EXISTS populist_url TEXT;
