-- Add down migration script here
ALTER TABLE politician
DROP COLUMN legiscan_people_id,
DROP COLUMN crp_candidate_id,
DROP COLUMN fec_candidate_id;