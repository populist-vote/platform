-- Add down migration script here
DROP INDEX IF EXISTS idx_statement_view_statement_id;
DROP INDEX IF EXISTS idx_statement_view_session_id;
DROP TABLE IF EXISTS statement_view;
