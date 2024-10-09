-- Add down migration script here
ALTER TYPE organization_role_type RENAME VALUE 'read_only' TO 'read-only';
