-- Add down migration script here
ALTER TABLE politician DROP COLUMN party_id;
ALTER TABLE race DROP COLUMN party_id;
ALTER TABLE user_profile DROP COLUMN party_id;

DROP TABLE party;
