-- Add down migration script here
DROP TRIGGER IF EXISTS author_on_inserted_politician ON politician;
DROP FUNCTION IF EXISTS create_author_from_politician;
