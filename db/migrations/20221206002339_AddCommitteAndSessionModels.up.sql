-- Add up migration script here
CREATE TABLE IF NOT EXISTS committee (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    state state,
    chair_id uuid REFERENCES politician(id),
    created_at timestamptz  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz  NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS subcommittee (
    committee_id uuid NOT NULL REFERENCES committee(id)
) INHERITS (committee);

CREATE TABLE session (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    name TEXT NOT NULL,  -- e.g. 2021-2022 Special Session
    description TEXT NOT NULL,
    state state,
    start_date date,
    end_date date,
    congress_name TEXT NOT NULL,  -- e.g. "73rd General Assembly"
    created_at timestamptz  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz  NOT NULL DEFAULT CURRENT_TIMESTAMP
);