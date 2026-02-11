-- SQL Query to compare ingest_staging.stg_mn_politicians with politician table
-- to find similar matching names

-- Option 1: Using fuzzystrmatch similarity (requires fuzzystrmatch extension)
-- This compares first_name and last_name similarity
SELECT 
    stg.id AS staging_id,
    stg.slug AS staging_slug,
    stg.first_name AS staging_first_name,
    stg.middle_name AS staging_middle_name,
    stg.last_name AS staging_last_name,
    stg.suffix AS staging_suffix,
    stg.full_name AS staging_full_name,
    prod.id AS production_id,
    prod.slug AS production_slug,
    prod.first_name AS production_first_name,
    prod.middle_name AS production_middle_name,
    prod.last_name AS production_last_name,
    prod.suffix AS production_suffix,
    prod.full_name AS production_full_name,
    -- Calculate similarity scores
    similarity(
        LOWER(stg.first_name || ' ' || stg.last_name),
        LOWER(prod.first_name || ' ' || prod.last_name)
    ) AS name_similarity,
    similarity(
        LOWER(COALESCE(stg.first_name, '') || ' ' || COALESCE(stg.middle_name, '') || ' ' || stg.last_name),
        LOWER(COALESCE(prod.first_name, '') || ' ' || COALESCE(prod.middle_name, '') || ' ' || prod.last_name)
    ) AS full_name_similarity,
    -- Check if slugs match (exact match)
    CASE WHEN stg.slug = prod.slug THEN true ELSE false END AS slug_match
FROM ingest_staging.stg_mn_politicians stg
CROSS JOIN politician prod
WHERE 
    -- Filter for similar names (similarity > 0.5 means 50% similar)
    similarity(
        LOWER(stg.first_name || ' ' || stg.last_name),
        LOWER(prod.first_name || ' ' || prod.last_name)
    ) > 0.5
    -- Or if slugs match exactly
    OR stg.slug = prod.slug
ORDER BY 
    name_similarity DESC,
    stg.last_name,
    stg.first_name;

-- Option 2: More comprehensive comparison with multiple matching strategies
-- This includes exact matches, slug matches, and similarity matches
WITH matches AS (
    SELECT 
        stg.id AS staging_id,
        stg.slug AS staging_slug,
        stg.ref_key AS staging_ref_key,
        stg.first_name AS staging_first_name,
        stg.middle_name AS staging_middle_name,
        stg.last_name AS staging_last_name,
        stg.suffix AS staging_suffix,
        stg.full_name AS staging_full_name,
        stg.email AS staging_email,
        stg.phone AS staging_phone,
        prod.id AS production_id,
        prod.slug AS production_slug,
        prod.ref_key AS production_ref_key,
        prod.first_name AS production_first_name,
        prod.middle_name AS production_middle_name,
        prod.last_name AS production_last_name,
        prod.suffix AS production_suffix,
        prod.full_name AS production_full_name,
        prod.email AS production_email,
        prod.phone AS production_phone,
        -- Similarity scores (first + middle + last name)
        similarity(
            LOWER(COALESCE(stg.first_name, '') || ' ' || COALESCE(stg.middle_name, '') || ' ' || stg.last_name),
            LOWER(COALESCE(prod.first_name, '') || ' ' || COALESCE(prod.middle_name, '') || ' ' || prod.last_name)
        ) AS name_similarity,
        similarity(
            LOWER(COALESCE(stg.first_name, '') || ' ' || COALESCE(stg.middle_name, '') || ' ' || stg.last_name),
            LOWER(COALESCE(prod.first_name, '') || ' ' || COALESCE(prod.middle_name, '') || ' ' || prod.last_name)
        ) AS full_name_similarity,
        -- Match type indicators
        CASE WHEN stg.email = prod.email AND stg.email IS NOT NULL AND prod.email IS NOT NULL THEN 'exact_email'
             WHEN stg.phone = prod.phone AND stg.phone IS NOT NULL AND prod.phone IS NOT NULL AND stg.phone <> '' AND prod.phone <> '' THEN 'exact_phone'
             WHEN LOWER(stg.first_name) = LOWER(prod.first_name) 
                  AND LOWER(stg.last_name) = LOWER(prod.last_name) 
                  AND (
                      (COALESCE(TRIM(stg.middle_name), '') = '' AND COALESCE(TRIM(prod.middle_name), '') = '')
                      OR (COALESCE(TRIM(stg.middle_name), '') <> '' AND COALESCE(TRIM(prod.middle_name), '') <> '' AND LOWER(TRIM(stg.middle_name)) = LOWER(TRIM(prod.middle_name)))
                  )
             THEN 'exact_name'
             WHEN stg.slug = prod.slug THEN 'exact_slug' 
             WHEN similarity(
                 LOWER(COALESCE(stg.first_name, '') || ' ' || COALESCE(stg.middle_name, '') || ' ' || stg.last_name),
                 LOWER(COALESCE(prod.first_name, '') || ' ' || COALESCE(prod.middle_name, '') || ' ' || prod.last_name)
             ) > 0.8 THEN 'high_similarity'
             WHEN similarity(
                 LOWER(COALESCE(stg.first_name, '') || ' ' || COALESCE(stg.middle_name, '') || ' ' || stg.last_name),
                 LOWER(COALESCE(prod.first_name, '') || ' ' || COALESCE(prod.middle_name, '') || ' ' || prod.last_name)
             ) > 0.6 THEN 'medium_similarity'
             ELSE 'low_similarity'
        END AS match_type
    FROM ingest_staging.stg_mn_politicians stg
    CROSS JOIN politician prod
    WHERE 
        -- Exact matches
        stg.slug = prod.slug
        OR (stg.email = prod.email AND stg.email IS NOT NULL AND prod.email IS NOT NULL)
        OR (stg.phone = prod.phone AND stg.phone IS NOT NULL AND prod.phone IS NOT NULL AND stg.phone <> '' AND prod.phone <> '')
        OR (
            LOWER(stg.first_name) = LOWER(prod.first_name) 
            AND LOWER(stg.last_name) = LOWER(prod.last_name)
            AND (
                (COALESCE(TRIM(stg.middle_name), '') = '' AND COALESCE(TRIM(prod.middle_name), '') = '')
                OR (COALESCE(TRIM(stg.middle_name), '') <> '' AND COALESCE(TRIM(prod.middle_name), '') <> '' AND LOWER(TRIM(stg.middle_name)) = LOWER(TRIM(prod.middle_name)))
            )
        )
        -- Similarity matches (first + middle + last, threshold: 0.6 = 60% similar)
        OR similarity(
            LOWER(COALESCE(stg.first_name, '') || ' ' || COALESCE(stg.middle_name, '') || ' ' || stg.last_name),
            LOWER(COALESCE(prod.first_name, '') || ' ' || COALESCE(prod.middle_name, '') || ' ' || prod.last_name)
        ) > 0.6
)
SELECT *
FROM matches
ORDER BY 
    CASE match_type
        WHEN 'exact_email' THEN 1
        WHEN 'exact_phone' THEN 2
        WHEN 'exact_name' THEN 3
        WHEN 'exact_slug' THEN 4
        WHEN 'high_similarity' THEN 5
        WHEN 'medium_similarity' THEN 6
        ELSE 7
    END,
    name_similarity DESC,
    staging_last_name,
    staging_first_name;

-- Option 3: Summary view showing match counts by type
SELECT 
    CASE 
        WHEN stg.email = prod.email AND stg.email IS NOT NULL AND prod.email IS NOT NULL THEN 'Exact Email Match'
        WHEN stg.phone = prod.phone AND stg.phone IS NOT NULL AND prod.phone IS NOT NULL AND stg.phone <> '' AND prod.phone <> '' THEN 'Exact Phone Match'
        WHEN LOWER(stg.first_name) = LOWER(prod.first_name) 
             AND LOWER(stg.last_name) = LOWER(prod.last_name)
             AND (
                 (COALESCE(TRIM(stg.middle_name), '') = '' AND COALESCE(TRIM(prod.middle_name), '') = '')
                 OR (COALESCE(TRIM(stg.middle_name), '') <> '' AND COALESCE(TRIM(prod.middle_name), '') <> '' AND LOWER(TRIM(stg.middle_name)) = LOWER(TRIM(prod.middle_name)))
             ) THEN 'Exact Name Match'
        WHEN stg.slug = prod.slug THEN 'Exact Slug Match'
        WHEN similarity(
            LOWER(COALESCE(stg.first_name, '') || ' ' || COALESCE(stg.middle_name, '') || ' ' || stg.last_name),
            LOWER(COALESCE(prod.first_name, '') || ' ' || COALESCE(prod.middle_name, '') || ' ' || prod.last_name)
        ) > 0.8 THEN 'High Similarity (>80%)'
        WHEN similarity(
            LOWER(COALESCE(stg.first_name, '') || ' ' || COALESCE(stg.middle_name, '') || ' ' || stg.last_name),
            LOWER(COALESCE(prod.first_name, '') || ' ' || COALESCE(prod.middle_name, '') || ' ' || prod.last_name)
        ) > 0.6 THEN 'Medium Similarity (60-80%)'
        ELSE 'Low Similarity (<60%)'
    END AS match_category,
    COUNT(*) AS match_count
FROM ingest_staging.stg_mn_politicians stg
CROSS JOIN politician prod
WHERE 
    stg.slug = prod.slug
    OR (stg.email = prod.email AND stg.email IS NOT NULL AND prod.email IS NOT NULL)
    OR (stg.phone = prod.phone AND stg.phone IS NOT NULL AND prod.phone IS NOT NULL AND stg.phone <> '' AND prod.phone <> '')
    OR (
        LOWER(stg.first_name) = LOWER(prod.first_name) 
        AND LOWER(stg.last_name) = LOWER(prod.last_name)
        AND (
            (COALESCE(TRIM(stg.middle_name), '') = '' AND COALESCE(TRIM(prod.middle_name), '') = '')
            OR (COALESCE(TRIM(stg.middle_name), '') <> '' AND COALESCE(TRIM(prod.middle_name), '') <> '' AND LOWER(TRIM(stg.middle_name)) = LOWER(TRIM(prod.middle_name)))
        )
    )
    OR similarity(
        LOWER(COALESCE(stg.first_name, '') || ' ' || COALESCE(stg.middle_name, '') || ' ' || stg.last_name),
        LOWER(COALESCE(prod.first_name, '') || ' ' || COALESCE(prod.middle_name, '') || ' ' || prod.last_name)
    ) > 0.6
GROUP BY match_category
ORDER BY 
    CASE match_category
        WHEN 'Exact Email Match' THEN 1
        WHEN 'Exact Phone Match' THEN 2
        WHEN 'Exact Name Match' THEN 3
        WHEN 'Exact Slug Match' THEN 4
        WHEN 'High Similarity (>80%)' THEN 5
        WHEN 'Medium Similarity (60-80%)' THEN 6
        ELSE 7
    END;

-- Option 4: Search a name in staging and production; show offices (via race_candidates -> race -> office) side by side
-- Change the search term in the search_name CTE below. Matches common shortened names (e.g. Andy<->Andrew, Greg<->Gregory).
WITH search_name AS (
    SELECT 'Smith'::text AS name  -- Change this to search for a different name
),
-- Add nickname pairs to the nickname_pairs CTE below, in the order of shorter <> longer.
nickname_pairs AS (
    SELECT * FROM (VALUES
        ('Andy', 'Andrew'),
        ('Greg', 'Gregory'),
        ('Chris', 'Christopher'),
        ('Chris', 'Christian'),
        ('Chris', 'Christoph'),
        ('Christoph', 'Christopher'),
        ('Rob', 'Robert'),
        ('Rob', 'Robby'),
        ('Robby', 'Robert'),
        ('Bob', 'Robert'),
        ('Bob', 'Bobby'),
        ('Bobby', 'Robert'),
        ('Joe', 'Joseph'),
        ('Joe', 'Joey'),
        ('Joey', 'Joseph'),
        ('Jim', 'James'),
        ('Jim', 'Jimmy'),
        ('Jimmy', 'James'),
        ('Mike', 'Michael'),
        ('Ben', 'Benjamin'),
        ('Dave', 'David'),
        ('Dan', 'Daniel'),
        ('Dan', 'Danny'),
        ('Danny', 'Daniel'),
        ('Liz', 'Elizabeth'),
        ('Liz', 'Lizzie'),
        ('Lizzie', 'Elizabeth'),
        ('Deb', 'Deborah'),
        ('Deb', 'Debbie'),
        ('Debbie', 'Deborah')
    ) AS t(short_form, long_form)
),
search_variants AS (
    SELECT DISTINCT variant FROM (
        SELECT (SELECT name FROM search_name) AS variant
        UNION ALL
        SELECT REGEXP_REPLACE((SELECT name FROM search_name), '(?i)' || short_form, long_form)
        FROM nickname_pairs
        WHERE (SELECT name FROM search_name) ~* short_form
        UNION ALL
        SELECT REGEXP_REPLACE((SELECT name FROM search_name), '(?i)' || long_form, short_form)
        FROM nickname_pairs
        WHERE (SELECT name FROM search_name) ~* long_form
    ) AS v
),
staging_offices AS (
    SELECT
        stg.id AS politician_id,
        stg.slug AS politician_slug,
        stg.full_name AS full_name,
        string_agg(DISTINCT so.slug, ' | ' ORDER BY so.slug) AS office_slugs,
        string_agg(DISTINCT so.county, ' | ' ORDER BY so.county) AS office_counties,
        string_agg(DISTINCT e.election_date::text, ' | ' ORDER BY e.election_date::text) AS election_dates
    FROM ingest_staging.stg_mn_politicians stg
    CROSS JOIN search_name sn
    LEFT JOIN ingest_staging.stg_mn_race_candidates src ON src.candidate_id = stg.id
    LEFT JOIN ingest_staging.stg_mn_races sr ON sr.id = src.race_id
    LEFT JOIN ingest_staging.stg_mn_offices so ON so.id = sr.office_id
    LEFT JOIN election e ON e.id = sr.election_id
    WHERE EXISTS (SELECT 1 FROM search_variants sv WHERE stg.full_name ILIKE '%' || sv.variant || '%')
    GROUP BY stg.id, stg.slug, stg.full_name
),
prod_offices AS (
    SELECT
        p.id AS politician_id,
        p.slug AS politician_slug,
        p.full_name AS full_name,
        string_agg(DISTINCT o.slug, ' | ' ORDER BY o.slug) AS office_slugs,
        string_agg(DISTINCT o.county, ' | ' ORDER BY o.county) AS office_counties,
        string_agg(DISTINCT e.election_date::text, ' | ' ORDER BY e.election_date::text) AS election_dates,
        MAX(current_office.slug) AS current_office_slug
    FROM politician p
    CROSS JOIN search_name sn
    LEFT JOIN race_candidates rc ON rc.candidate_id = p.id
    LEFT JOIN race r ON r.id = rc.race_id
    LEFT JOIN office o ON o.id = r.office_id
    LEFT JOIN election e ON e.id = r.election_id
    LEFT JOIN office current_office ON current_office.id = p.office_id
    WHERE EXISTS (SELECT 1 FROM search_variants sv WHERE p.full_name ILIKE '%' || sv.variant || '%')
    GROUP BY p.id, p.slug, p.full_name
)
SELECT
    (SELECT name FROM search_name) AS search_term,
    stg.politician_slug AS staging_slug,
    stg.full_name AS staging_full_name,
    stg.office_slugs AS staging_office_slugs,
    stg.office_counties AS staging_office_counties,
    stg.election_dates AS staging_election_dates,
    prod.politician_slug AS prod_slug,
    prod.full_name AS prod_full_name,
    prod.current_office_slug AS prod_current_office_slug,
    prod.office_slugs AS prod_office_slugs,
    prod.office_counties AS prod_office_counties,
    prod.election_dates AS prod_election_dates
FROM staging_offices stg
FULL OUTER JOIN prod_offices prod ON prod.politician_slug = stg.politician_slug
ORDER BY COALESCE(stg.full_name, prod.full_name);

