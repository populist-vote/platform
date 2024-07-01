-- Add down migration script here
ALTER TABLE question_submission DROP COLUMN is_locked;
