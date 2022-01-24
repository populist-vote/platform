-- Add down migration script here
ALTER TABLE issue_tag
DROP COLUMN category;