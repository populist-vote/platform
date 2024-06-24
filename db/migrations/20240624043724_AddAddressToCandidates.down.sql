-- Add down migration script here
ALTER TABLE politician
DROP COLUMN residence_address_id,
DROP COLUMN campaign_address_id;
