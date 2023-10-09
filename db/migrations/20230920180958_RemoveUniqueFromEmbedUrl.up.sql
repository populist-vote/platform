-- Add up migration script here
ALTER TABLE embed_origin DROP CONSTRAINT IF EXISTS embed_origin_url_key;
