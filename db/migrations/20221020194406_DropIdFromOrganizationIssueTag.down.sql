-- Add down migration script here
ALTER TABLE organization_issue_tags
ADD COLUMN id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
DROP CONSTRAINT organization_issue_tags_pkey;