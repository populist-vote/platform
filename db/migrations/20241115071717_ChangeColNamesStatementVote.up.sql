-- Add up migration script here
ALTER TABLE statement_vote
RENAME COLUMN participant_id TO user_id;

ALTER TABLE statement_vote
ADD COLUMN session_id UUID;
