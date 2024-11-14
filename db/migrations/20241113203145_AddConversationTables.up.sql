-- Add up migration script here

CREATE TABLE IF NOT EXISTS conversation (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    prompt TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER set_updated_at
BEFORE UPDATE
ON conversation
FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();

CREATE TABLE IF NOT EXISTS statement (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversation (id),
    content TEXT NOT NULL,
    author_id UUID REFERENCES populist_user (id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TRIGGER set_updated_at
BEFORE UPDATE
ON statement
FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();

CREATE TABLE IF NOT EXISTS statement_vote (
    id UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    statement_id UUID REFERENCES statement (id),
    participant_id UUID REFERENCES populist_user (id),
    vote_type ARGUMENT_POSITION NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (statement_id, participant_id)
);
