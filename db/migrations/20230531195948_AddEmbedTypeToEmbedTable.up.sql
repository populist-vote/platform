-- Add up migration script here
CREATE TYPE embed_type AS ENUM ('legislation', 'politician', 'poll', 'question');

ALTER TABLE embed 
ADD COLUMN embed_type embed_type;

UPDATE embed SET embed_type = (attributes->>'embedType')::embed_type;

ALTER TABLE embed
ALTER COLUMN embed_type
SET NOT NULL;