-- Add up migration script here
ALTER TABLE poll_submission
ALTER COLUMN poll_option_id DROP NOT NULL,
ADD CONSTRAINT chk_poll_option_or_write_in
CHECK (
    (poll_option_id IS NOT NULL AND write_in_response IS NULL)
    OR (poll_option_id IS NULL AND write_in_response IS NOT NULL)
);
