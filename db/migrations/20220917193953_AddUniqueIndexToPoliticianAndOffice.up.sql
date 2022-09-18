-- Add up migration script here
CREATE UNIQUE INDEX IF NOT EXISTS politician_identifier ON politician (id, slug);
CREATE UNIQUE INDEX IF NOT EXISTS office_identifier ON office (id, slug);
CREATE UNIQUE INDEX IF NOT EXISTS race_identifier ON race (id, slug);