-- Add up migration script here
ALTER TABLE question
ADD COLUMN translations JSONB;

ALTER TABLE question_submission
ADD COLUMN translations JSONB;
