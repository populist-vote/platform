-- Add up migration script here
ALTER TABLE question_submission
ADD COLUMN copied_from_id UUID,
DROP COLUMN is_locked;
