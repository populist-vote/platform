-- Add up migration script here
CREATE TABLE IF NOT EXISTS question (
    id uuid NOT NULL PRIMARY KEY,
    prompt TEXT NOT NULL,
    response_char_limit INTEGER,
    response_placeholder_text TEXT,
    embed_id UUID REFERENCES embed(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE respondent (
    id uuid NOT NULL PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS question_submission (
    id uuid NOT NULL PRIMARY KEY,
    question_id UUID NOT NULL REFERENCES question(id) ON DELETE CASCADE,
    respondent_id UUID NOT NULL REFERENCES respondent(id) ON DELETE CASCADE,
    response TEXT NOT NULL,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS poll (
    id uuid NOT NULL PRIMARY KEY,
    name TEXT,
    prompt TEXT NOT NULL,
    embed_id UUID REFERENCES embed(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS poll_option (
    id uuid NOT NULL PRIMARY KEY,
    poll_id UUID NOT NULL REFERENCES poll(id) ON DELETE CASCADE,
    option_text TEXT NOT NULL,
    is_write_in BOOLEAN NOT NULL DEFAULT FALSE,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS poll_submission (
    id uuid NOT NULL PRIMARY KEY,
    poll_id UUID NOT NULL REFERENCES poll(id) ON DELETE CASCADE,
    poll_option_id UUID NOT NULL REFERENCES poll_option(id) ON DELETE CASCADE,
    respondent_id UUID NOT NULL REFERENCES respondent(id) ON DELETE CASCADE,
    write_in_response TEXT,
    created_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);
