-- Add up migration script here

ALTER TABLE bill
ADD COLUMN state state, 
ADD COLUMN legiscan_session_id INT,
ADD COLUMN legiscan_committee_id INT,
ADD COLUMN legiscan_committee TEXT,
ADD COLUMN legiscan_last_action TEXT,
ADD COLUMN legiscan_last_action_date DATE;