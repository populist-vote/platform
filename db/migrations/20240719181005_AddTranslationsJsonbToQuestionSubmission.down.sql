-- Add down migration script here
ALTER TABLE question
DROP COLUMN translations;

ALTER TABLE question_submission
DROP COLUMN translations;
