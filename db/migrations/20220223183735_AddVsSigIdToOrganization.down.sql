-- Add down migration script here
ALTER TABLE organization
DROP COLUMN votesmart_sig_id,
DROP COLUMN headquarters_address_id;