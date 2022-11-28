-- Add up migration script here
ALTER TABLE politician
ADD CONSTRAINT unique_politician_legiscan_id UNIQUE (legiscan_people_id),
ADD CONSTRAINT unique_politician_votesmart_id UNIQUE (votesmart_candidate_id);