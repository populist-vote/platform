-- Add down migration script here
ALTER TABLE bill
DROP COLUMN session_id,
DROP COLUMN committee_id,
ADD COLUMN legiscan_session_id INTEGER;

ALTER TABLE session
DROP COLUMN legiscan_session_id;

ALTER TABLE committee
DROP COLUMN legiscan_committee_id;