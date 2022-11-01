-- Add up migration script here
CREATE TABLE organization_politician_notes (
    id              uuid        NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    organization_id uuid        NOT NULL REFERENCES organization(id),
    politician_id   uuid        NOT NULL REFERENCES politician(id),
    election_id     uuid        NOT NULL REFERENCES election(id),
    notes           jsonb       NOT NULL DEFAULT '{}'::jsonb,
    issue_tag_ids   uuid[]      NOT NULL DEFAULT '{}'::uuid[],
    created_at      timestamptz NOT NULL DEFAULT NOW(),
    updated_at      timestamptz NOT NULL DEFAULT NOW()
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON organization_politician_notes
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();