-- Add up migration script here
ALTER TABLE issue_tag
ADD COLUMN category TEXT;