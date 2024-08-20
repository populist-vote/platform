ALTER TABLE question ALTER COLUMN organization_id DROP NOT NULL;
ALTER TABLE poll ALTER COLUMN organization_id DROP NOT NULL;

ALTER TABLE question DROP COLUMN organization_id;
ALTER TABLE poll DROP COLUMN organization_id;
