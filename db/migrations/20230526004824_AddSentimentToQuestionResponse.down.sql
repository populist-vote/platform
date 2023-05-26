-- Add down migration script here
ALTER TABLE question_submission
DROP COLUMN sentiment;

DROP TYPE IF EXISTS sentiment;

