-- Add up migration script here
CREATE TYPE vote_type AS ENUM ('plurality', 'ranked_choice');

ALTER TABLE race ADD COLUMN vote_type vote_type NOT NULL DEFAULT 'plurality';
