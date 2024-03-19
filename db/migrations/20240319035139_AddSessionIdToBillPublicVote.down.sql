-- Add down migration script here
ALTER TABLE bill_public_votes DROP COLUMN session_id;
DROP INDEX IF EXISTS bill_public_votes_session_id_idx;
