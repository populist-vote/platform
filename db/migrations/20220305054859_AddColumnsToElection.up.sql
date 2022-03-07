-- Add up migration script here
ALTER TABLE election
ADD COLUMN state state,
ADD COLUMN created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
ADD COLUMN updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc');