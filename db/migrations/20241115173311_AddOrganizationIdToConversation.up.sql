-- Add up migration script here
ALTER TABLE conversation
ADD COLUMN organization_id UUID REFERENCES organization (id);

ALTER TABLE conversation
RENAME COLUMN prompt TO topic;

UPDATE conversation SET
    organization_id = (SELECT id FROM organization WHERE slug = 'populist');

ALTER TABLE conversation
ALTER COLUMN organization_id SET NOT NULL;
