-- Add down migration script here
ALTER TABLE office
DROP COLUMN subtitle,
DROP COLUMN subtitle_short;