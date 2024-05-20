-- Add down migration script here
ALTER TABLE candidate_guide_questions
DROP CONSTRAINT unique_candidate_guide_question;
