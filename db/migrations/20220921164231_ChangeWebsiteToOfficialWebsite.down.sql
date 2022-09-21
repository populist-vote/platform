-- Add down migration script here
ALTER TABLE politician
RENAME COLUMN official_website_url TO website_url;