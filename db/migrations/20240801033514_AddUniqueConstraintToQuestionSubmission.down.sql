-- Add down migration script here
ALTER TABLE question_submission
DROP CONSTRAINT unique_question_submission;
