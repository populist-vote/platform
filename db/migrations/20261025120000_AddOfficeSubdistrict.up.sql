BEGIN;

ALTER TABLE office
ADD COLUMN subdistrict TEXT;

-- A school or hospital value plus a district: the district is the subdivision.
UPDATE office
SET
    subdistrict = district,
    district = COALESCE(NULLIF(btrim(school_district), ''), NULLIF(btrim(hospital_district), ''))
WHERE NULLIF(btrim(district), '') IS NOT NULL
  AND (
    NULLIF(btrim(school_district), '') IS NOT NULL
    OR NULLIF(btrim(hospital_district), '') IS NOT NULL
  )
  AND btrim(district) IS DISTINCT FROM COALESCE(
    NULLIF(btrim(school_district), ''),
    NULLIF(btrim(hospital_district), '')
  );

-- A school or hospital value with no district: it becomes the district.
UPDATE office
SET district = COALESCE(NULLIF(btrim(school_district), ''), NULLIF(btrim(hospital_district), ''))
WHERE NULLIF(btrim(subdistrict), '') IS NULL
  AND NULLIF(btrim(district), '') IS NULL
  AND (
    NULLIF(btrim(school_district), '') IS NOT NULL
    OR NULLIF(btrim(hospital_district), '') IS NOT NULL
  );

COMMIT;
