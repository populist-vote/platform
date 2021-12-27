-- Add up migration script here
ALTER TABLE politician
ADD COLUMN legiscan_people_id INT,
ADD COLUMN crp_candidate_id VARCHAR,
ADD COLUMN fec_candidate_id VARCHAR;
