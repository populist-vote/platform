-- Add down migration script here
ALTER TABLE address
DROP CONSTRAINT unique_address;
