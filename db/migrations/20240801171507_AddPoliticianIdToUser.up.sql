-- Add up migration script here
ALTER TABLE organization
ADD COLUMN politician_id UUID REFERENCES politician (id);

CREATE INDEX idx_organization_politician_id ON organization (politician_id);

CREATE TABLE invite_token (
    token UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL,
    organization_id UUID REFERENCES organization (id),
    politician_id UUID REFERENCES politician (id),
    role ORGANIZATION_ROLE_TYPE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    expires_at TIMESTAMP NOT NULL,
    sent_at TIMESTAMP,
    accepted_at TIMESTAMP,
    invited_by UUID REFERENCES populist_user (id),
    -- number of times the token can be used, NULL means unlimited
    invite_limit INT,
    -- Ensure uniqueness per role in organization
    CONSTRAINT unique_invite_per_role UNIQUE (email, organization_id, role),
    CONSTRAINT unique_invite_per_politician UNIQUE (email, politician_id)

);

CREATE INDEX idx_invite_email ON invite_token (email);
CREATE INDEX idx_invite_org ON invite_token (organization_id);
CREATE INDEX idx_invite_politician ON invite_token (politician_id);
