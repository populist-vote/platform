-- Add up migration script here
ALTER TABLE question_submission
ADD COLUMN editorial TEXT;
