BEGIN;

-- Pads each number so "District 2" sorts before "District 10".
CREATE OR REPLACE FUNCTION natural_sort_key(value text)
RETURNS text
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
AS $$
    SELECT CASE
        WHEN value IS NULL OR btrim(value) = '' THEN NULL
        ELSE (
            SELECT string_agg(
                CASE
                    WHEN part[1] IS NOT NULL THEN lpad(part[1], 12, '0')
                    ELSE lower(part[2])
                END,
                ''
                ORDER BY ordinality
            )
            FROM regexp_matches(value, '(\d+)|(\D+)', 'g')
                WITH ORDINALITY AS parts(part, ordinality)
        )
    END;
$$;

COMMIT;
