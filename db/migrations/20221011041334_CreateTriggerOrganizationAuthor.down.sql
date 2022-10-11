-- Add down migration script here
DROP TRIGGER IF EXISTS author_on_inserted_organization ON organization;