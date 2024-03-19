-- Add up migration script here
ALTER TABLE bill_public_votes
ADD COLUMN session_id UUID;

CREATE UNIQUE INDEX bill_public_votes_session_id_idx ON bill_public_votes (
    bill_id, session_id
);
