-- Add up migration script here
ALTER TABLE politician ADD COLUMN display_name TEXT NOT NULL DEFAULT '';
UPDATE politician SET
    display_name = COALESCE(preferred_name, first_name) || ' ' || last_name;
ALTER TABLE politician ALTER COLUMN first_name DROP NOT NULL;
ALTER TABLE politician ALTER COLUMN last_name DROP NOT NULL;
