-- Add up migration script here
ALTER TABLE bill 
DROP COLUMN legiscan_session_id,
ADD COLUMN session_id UUID REFERENCES session(id),
ADD COLUMN committee_id UUID REFERENCES committee(id);

ALTER TABLE session
ADD COLUMN legiscan_session_id INTEGER;

ALTER TABLE committee
ADD COLUMN legiscan_committee_id INTEGER;
