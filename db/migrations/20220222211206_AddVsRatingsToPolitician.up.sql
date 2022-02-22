-- Add up migration script here
ALTER TABLE politician
    ADD COLUMN votesmart_candidate_ratings JSONB NOT NULL DEFAULT '[]'::jsonb;