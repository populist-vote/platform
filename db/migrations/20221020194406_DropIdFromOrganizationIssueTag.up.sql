-- Add up migration script here
ALTER TABLE organization_issue_tags 
DROP COLUMN id,
ADD PRIMARY KEY (organization_id, issue_tag_id);