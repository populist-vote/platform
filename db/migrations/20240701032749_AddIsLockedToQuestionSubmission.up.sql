-- Add up migration script here
ALTER TABLE question_submission
ADD COLUMN is_locked boolean NOT NULL DEFAULT false;
