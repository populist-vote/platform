CREATE TABLE user_api_key (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES populist_user(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL CHECK (length(BTRIM(name)) > 0),
    key_prefix VARCHAR(16) NOT NULL,
    key_hash BYTEA NOT NULL UNIQUE CHECK (octet_length(key_hash) = 32),
    last_used_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX user_api_key_active_name_idx
    ON user_api_key (user_id, LOWER(name))
    WHERE revoked_at IS NULL;

CREATE INDEX user_api_key_active_hash_idx
    ON user_api_key (key_hash)
    WHERE revoked_at IS NULL;
