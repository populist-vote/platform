-- Add up migration script here

ALTER TABLE question_submission
ADD CONSTRAINT fk_question_submissions_candidate_id
FOREIGN KEY (candidate_id)
REFERENCES politician (id);
