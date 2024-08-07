-- Add up migration script here
ALTER TABLE invite_token
DROP CONSTRAINT invite_token_invited_by_fkey,
ADD CONSTRAINT invite_token_invited_by_fkey
FOREIGN KEY (invited_by)
REFERENCES populist_user (id)
ON DELETE CASCADE;
