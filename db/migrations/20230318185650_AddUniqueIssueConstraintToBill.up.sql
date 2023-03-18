-- Add up migration script here
ALTER TABLE bill_issue_tags ADD CONSTRAINT unique_issue UNIQUE (bill_id, issue_tag_id);