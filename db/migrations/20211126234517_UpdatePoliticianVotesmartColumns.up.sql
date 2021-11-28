-- Add up migration script here
ALTER TABLE politician
    RENAME COLUMN vote_smart_candidate_bio TO votesmart_candidate_bio;

ALTER TABLE politician 
    RENAME COLUMN vote_smart_candidate_id TO votesmart_candidate_id;

ALTER TABLE politician    
    ALTER COLUMN votesmart_candidate_id TYPE INT USING (trim(votesmart_candidate_id)::integer);

ALTER TABLE politician
    ADD CONSTRAINT votesmart_unique UNIQUE (votesmart_candidate_id);

ALTER TABLE politician   
    ADD UNIQUE (id, votesmart_candidate_id);

ALTER TYPE political_party ADD VALUE 'unknown';

