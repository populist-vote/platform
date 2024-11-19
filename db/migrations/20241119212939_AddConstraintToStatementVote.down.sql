-- Add down migration script here
ALTER TABLE statement_vote
DROP CONSTRAINT unique_user_statement,
DROP CONSTRAINT unique_session_statement;
