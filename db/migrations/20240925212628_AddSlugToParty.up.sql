-- Add up migration script here

ALTER TABLE party ADD COLUMN slug text;

UPDATE party SET slug = slugify(name);

ALTER TABLE party ALTER COLUMN slug SET NOT NULL;

ALTER TABLE party ADD CONSTRAINT party_slug_key UNIQUE (slug);