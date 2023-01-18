-- Add up migration script here
ALTER TABLE populist_user 
ADD COLUMN organization_id uuid REFERENCES organization(id) ON DELETE SET NULL;