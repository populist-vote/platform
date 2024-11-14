-- Add up migration script here
CREATE INDEX statement_content_search_idx ON statement USING gin (
    to_tsvector('english', content)
);

CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Create a GiST index for trigram matching
CREATE INDEX statement_content_trgm_idx ON statement USING gist (
    content gist_trgm_ops
);
