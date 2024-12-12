-- Add up migration script here
ALTER TABLE statement
DROP CONSTRAINT IF EXISTS statement_conversation_id_fkey,
ADD CONSTRAINT statement_conversation_id_fkey
FOREIGN KEY (conversation_id)
REFERENCES conversation (id)
ON DELETE CASCADE;

ALTER TABLE statement_vote
DROP CONSTRAINT IF EXISTS statement_vote_statement_id_fkey,
ADD CONSTRAINT statement_vote_statement_id_fkey
FOREIGN KEY (statement_id)
REFERENCES statement (id)
ON DELETE CASCADE;
