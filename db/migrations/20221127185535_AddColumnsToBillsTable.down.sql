-- Add down migration script here
ALTER TABLE bill
DROP COLUMN state,
DROP COLUMN legiscan_session_id,
DROP COLUMN legiscan_committee_id,
DROP COLUMN legiscan_committee,
DROP COLUMN legiscan_last_action,
DROP COLUMN legiscan_last_action_date;
