-- Add up migration script here
ALTER TABLE statement_vote
ADD CONSTRAINT unique_user_statement UNIQUE (user_id, statement_id),
ADD CONSTRAINT unique_session_statement UNIQUE (session_id, statement_id);
