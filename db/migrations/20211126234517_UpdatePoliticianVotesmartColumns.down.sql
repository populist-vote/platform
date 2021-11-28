-- Add down migration script here
ALTER TABLE politician
    ALTER COLUMN votesmart_candidate_id TYPE TEXT;

ALTER TABLE politician
    RENAME COLUMN votesmart_candidate_id TO vote_smart_candidate_id;

ALTER TABLE politician
    RENAME COLUMN votesmart_candidate_bio TO vote_smart_candidate_bio;

ALTER TABLE politician
    DROP CONSTRAINT votesmart_unique;

ALTER TABLE politician   
    DROP UNIQUE (id, votesmart_candidate_id);