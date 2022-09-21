-- Add up migration script here
ALTER TABLE politician
RENAME COLUMN website_url TO official_website_url;

ALTER TABLE office
ADD COLUMN name TEXT; 