-- Add up migration script here
ALTER TABLE embed
ADD COLUMN created_by uuid NOT NULL REFERENCES populist_user(id) ON DELETE CASCADE,
ADD COLUMN updated_by uuid NOT NULL REFERENCES populist_user(id) ON DELETE CASCADE;

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON embed
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();