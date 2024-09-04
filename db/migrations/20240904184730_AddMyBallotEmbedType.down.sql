-- Add down migration script here
CREATE TYPE embed_type_new AS ENUM (
    'legislation',
    'politician',
    'poll',
    'question',
    'race',
    'legislation_tracker',
    'candidate_guide'
);

ALTER TABLE embed ALTER COLUMN embed_type
TYPE embed_type_new USING embed_type::text::embed_type_new;

DROP TYPE embed_type CASCADE;
ALTER TYPE embed_type_new RENAME TO embed_type;
