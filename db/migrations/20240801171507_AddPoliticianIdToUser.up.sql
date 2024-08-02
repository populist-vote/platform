-- Add up migration script here
ALTER TABLE politician
ADD COLUMN organization_id UUID REFERENCES organization (id);

CREATE INDEX idx_organization_id ON politician (organization_id);

CREATE TABLE invite_token (
    token UUID PRIMARY KEY,
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
    invite_limit INT
);

CREATE INDEX idx_invite_token_organization_id ON invite_token (email);
