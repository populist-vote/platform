-- Add down migration script here
DROP TABLE IF EXISTS organization_user;

ALTER TABLE populist_user
ADD COLUMN organization_id UUID REFERENCES organization (id) ON DELETE CASCADE;

ALTER TABLE populist_user ADD COLUMN role USER_ROLE NOT NULL DEFAULT 'basic';
