-- Add down migration script here
ALTER TABLE politician
DROP CONSTRAINT unique_politician_legiscan_id,
DROP CONSTRAINT unique_politician_votesmart_id;