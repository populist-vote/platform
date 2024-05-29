-- Add down migration script here
ALTER TABLE question_submission
DROP CONSTRAINT fk_question_submissions_candidate_id;
