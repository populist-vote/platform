BEGIN;

UPDATE office
SET district = subdistrict
WHERE NULLIF(btrim(subdistrict), '') IS NOT NULL
  AND (
    NULLIF(btrim(school_district), '') IS NOT NULL
    OR NULLIF(btrim(hospital_district), '') IS NOT NULL
  );

UPDATE office
SET district = NULL
WHERE subdistrict IS NULL
  AND (
    district = NULLIF(btrim(school_district), '')
    OR district = NULLIF(btrim(hospital_district), '')
  );

ALTER TABLE office
DROP COLUMN subdistrict;

COMMIT;
