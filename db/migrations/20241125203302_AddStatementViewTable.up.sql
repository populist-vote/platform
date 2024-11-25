-- Add up migration script here
CREATE TABLE statement_view (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    statement_id UUID NOT NULL REFERENCES statement (id),
    session_id UUID NOT NULL,
    user_id UUID REFERENCES populist_user (id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (statement_id, session_id)
);

CREATE INDEX idx_statement_view_statement_id ON statement_view (statement_id);
CREATE INDEX idx_statement_view_session_id ON statement_view (session_id);
