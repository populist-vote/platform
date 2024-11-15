-- Add down migration script here
ALTER TABLE statement_vote
RENAME COLUMN user_id TO participant_id;

ALTER TABLE statement_vote
DROP COLUMN session_id;
