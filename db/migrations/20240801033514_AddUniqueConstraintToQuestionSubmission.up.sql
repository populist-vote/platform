-- Add up migration script here
ALTER TABLE question_submission
ADD CONSTRAINT unique_question_submission
UNIQUE (candidate_id, question_id);
