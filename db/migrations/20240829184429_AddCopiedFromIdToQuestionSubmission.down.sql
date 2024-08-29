-- Add down migration script here
ALTER TABLE question_submission
DROP COLUMN copied_from_id,
ADD COLUMN IF NOT EXISTS is_locked BOOLEAN DEFAULT FALSE;
