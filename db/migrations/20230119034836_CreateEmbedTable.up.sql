-- Add up migration script here
CREATE TABLE embed (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    organization_id uuid NOT NULL REFERENCES organization(id),
    name TEXT NOT NULL,
    description TEXT,
    populist_url TEXT NOT NULL,
    attributes JSONB NOT NULL DEFAULT '{}'::JSONB,
    created_at TIMESTAMPTZ  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ  NOT NULL DEFAULT CURRENT_TIMESTAMP
);