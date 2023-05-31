-- Add up migration script here
CREATE TABLE IF NOT EXISTS embed_origin (
    embed_id UUID NOT NULL REFERENCES embed(id) ON DELETE CASCADE,
    url TEXT NOT NULL UNIQUE,
    last_ping_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE UNIQUE INDEX IF NOT EXISTS embed_origin_identifier ON embed_origin (embed_id, url);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON embed_origin
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();