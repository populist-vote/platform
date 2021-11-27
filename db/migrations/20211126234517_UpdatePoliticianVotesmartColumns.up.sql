-- Add up migration script here
ALTER TABLE politician
    RENAME COLUMN votesmart_candidate_bio TO votesmart_candidate_bio;

ALTER TABLE politician    
    ALTER COLUMN vote_smart_candidate_id TYPE INT USING (trim(vote_smart_candidate_id)::integer);

ALTER TABLE politician 
    RENAME COLUMN vote_smart_candidate_id TO votesmart_candidate_id;