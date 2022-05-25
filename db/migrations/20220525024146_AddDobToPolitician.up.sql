-- Add up migration script here
ALTER TABLE politician
ADD COLUMN date_of_birth DATE;

UPDATE politician
SET date_of_birth = TO_DATE(votesmart_candidate_bio->'candidate'->>'birthDate', 'MM/DD/YYYY')
WHERE votesmart_candidate_bio->'candidate'->>'birthDate' LIKE '__/__/____';
