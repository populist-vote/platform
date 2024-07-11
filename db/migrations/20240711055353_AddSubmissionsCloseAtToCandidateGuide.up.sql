-- Add up migration script here
ALTER TABLE candidate_guide
ADD COLUMN submissions_open_at TIMESTAMPTZ,
ADD COLUMN submissions_close_at TIMESTAMPTZ;
