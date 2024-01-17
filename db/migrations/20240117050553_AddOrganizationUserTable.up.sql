-- Add up migration script here

CREATE TABLE IF NOT EXISTS organization_user (
    organization_id UUID NOT NULL REFERENCES organization (id),
    user_id UUID NOT NULL REFERENCES populist_user (id),
    role USER_ROLE NOT NULL DEFAULT 'basic',
    PRIMARY KEY (organization_id, user_id)
);

INSERT INTO organization_user (organization_id, user_id, role)
SELECT
    organization_id,
    id,
    role
FROM populist_user
WHERE organization_id IS NOT NULL;

ALTER TABLE populist_user DROP COLUMN IF EXISTS organization_id;
ALTER TABLE populist_user DROP COLUMN IF EXISTS role;
