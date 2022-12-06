-- Add down migration script here

DROP TABLE IF EXISTS subcommittee;
DROP TABLE IF EXISTS committee;
DROP TABLE IF EXISTS session;

ALTER TABLE bill_public_votes
DROP CONSTRAINT unique_bill_public_votes_bill_id_user_id,
DROP COLUMN created_at,
DROP COLUMN updated_at;