-- Add up migration script here
CREATE EXTENSION unaccent;

CREATE OR REPLACE FUNCTION generate_unique_slug(base_slug text, table_name text)
RETURNS text AS $$
DECLARE
    original_slug text;
    new_slug text;
    counter integer := 1;
    slug_exists boolean;
BEGIN
    -- Normalize base slug
    original_slug := LOWER(REGEXP_REPLACE(base_slug, '[^a-zA-Z0-9]+', '-', 'g'));
    new_slug := original_slug;

    LOOP
        -- Check if the slug exists in the table
        EXECUTE format('SELECT EXISTS (SELECT 1 FROM %I WHERE slug = %L)', table_name, new_slug)
        INTO slug_exists;

        -- If slug does not exist, return it
        IF NOT slug_exists THEN
            RETURN new_slug;
        END IF;

        -- Append or increment a number at the end of the slug
        IF counter = 1 THEN
            new_slug := original_slug || '-' || counter;
        ELSE
            new_slug := original_slug || '-' || counter;
        END IF;
        counter := counter + 1;
    END LOOP;
END;
$$ LANGUAGE plpgsql;
