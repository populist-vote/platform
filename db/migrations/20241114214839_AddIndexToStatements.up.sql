-- Add up migration script here
CREATE INDEX statement_content_search_idx ON statement USING gin (
    to_tsvector('english', content)
);
