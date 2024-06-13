CREATE OR REPLACE FUNCTION slugify(value TEXT) RETURNS TEXT AS $$
DECLARE
    result TEXT;
BEGIN
    -- Convert to lower case
    result := LOWER(value);

    -- Remove special characters except spaces and hyphens
    result := REGEXP_REPLACE(result, '[^a-z0-9 -]', '', 'g');

    -- Replace spaces with hyphens
    result := REGEXP_REPLACE(result, '\s+', '-', 'g');

    -- Replace multiple hyphens with a single hyphen
    result := REGEXP_REPLACE(result, '-+', '-', 'g');

    -- Trim hyphens from the beginning and end
    result := TRIM(BOTH '-' FROM result);

    RETURN result;
END;
$$ LANGUAGE plpgsql;
