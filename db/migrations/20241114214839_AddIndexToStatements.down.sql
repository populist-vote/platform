-- Add down migration script here
DROP INDEX IF EXISTS statement_content_search_idx;
DROP INDEX IF EXISTS statement_content_trgm_idx;
