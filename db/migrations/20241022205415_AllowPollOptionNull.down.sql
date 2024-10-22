-- Add down migration script here
ALTER TABLE poll_submission
ALTER COLUMN poll_option_id SET NOT NULL,
DROP CONSTRAINT chk_poll_option_or_write_in;
