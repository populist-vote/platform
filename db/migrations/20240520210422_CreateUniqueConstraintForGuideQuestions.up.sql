-- Add up migration script here
ALTER TABLE candidate_guide_questions
ADD CONSTRAINT unique_candidate_guide_question
UNIQUE (candidate_guide_id, question_id);
