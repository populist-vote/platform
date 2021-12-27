-- Add down migration script here
ALTER TABLE politician
DROP COLUMN legiscan_people_id;

ALTER TABLE politician
DROP COLUMN crp_candidate_id;