-- Add up migration script here
ALTER TABLE politician
ADD COLUMN full_name TEXT;

UPDATE politician
SET full_name = CONCAT(
    first_name,
    ' ',
    CASE
        WHEN
            preferred_name IS NOT NULL
            THEN CONCAT('(', preferred_name, ')')
        ELSE ''
    END,
    ' ',
    middle_name,
    ' ',
    last_name,
    ' ',
    suffix
);

ALTER TABLE politician
ALTER COLUMN full_name SET NOT NULL;
