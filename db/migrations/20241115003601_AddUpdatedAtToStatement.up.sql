-- Add up migration script here
ALTER TABLE statement ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT now();
