-- Add up migration script here
CREATE TYPE sentiment AS ENUM ('positive', 'negative', 'neutral', 'unknown');

ALTER TABLE question_submission 
ADD COLUMN sentiment sentiment;