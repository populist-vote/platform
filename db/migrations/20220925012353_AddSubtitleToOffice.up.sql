-- Add up migration script here
ALTER TABLE office
ADD COLUMN subtitle TEXT,
ADD COLUMN subtitle_short TEXT;