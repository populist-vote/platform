-- Add up migration script here
CREATE TABLE organization_respondents (
    organization_id UUID NOT NULL REFERENCES organization(id), 
    respondent_id UUID NOT NULL REFERENCES respondent(id) ON DELETE CASCADE,
    attributes JSONB NOT NULL DEFAULT '{}'::JSONB,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (organization_id, respondent_id)
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON organization_respondents
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();