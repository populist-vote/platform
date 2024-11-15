-- Add down migration script here
ALTER TABLE conversation
DROP COLUMN organization_id;

ALTER TABLE conversation
RENAME COLUMN topic TO prompt;
