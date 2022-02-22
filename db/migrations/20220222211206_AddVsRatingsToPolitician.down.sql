-- Add down migration script here
ALTER TABLE politician
    DROP COLUMN votesmart_candidate_ratings;