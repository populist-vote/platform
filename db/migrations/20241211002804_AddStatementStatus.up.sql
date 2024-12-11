-- Add up migration script here

CREATE TYPE statement_moderation_status AS ENUM (
    'unmoderated',
    'accepted',
    'rejected',
    'seed'
);

ALTER TABLE statement
-- First, add the new column as nullable
ADD COLUMN moderation_status statement_moderation_status;

-- Set default status for existing rows
UPDATE statement
SET moderation_status = 'unmoderated'::statement_moderation_status;

-- Make the column required
ALTER TABLE statement
ALTER COLUMN moderation_status SET NOT NULL,
ALTER COLUMN moderation_status
SET DEFAULT 'unmoderated'::statement_moderation_status;

-- Add index for performance
CREATE INDEX idx_statements_moderation_status ON statement (moderation_status);
