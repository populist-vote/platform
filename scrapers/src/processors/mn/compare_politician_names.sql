-- SQL Query to compare dbt_henry.stg_politicians with politician table
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
FROM dbt_henry.stg_politicians stg
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
        -- Similarity scores
        similarity(
            LOWER(stg.first_name || ' ' || stg.last_name),
            LOWER(prod.first_name || ' ' || prod.last_name)
        ) AS name_similarity,
        similarity(
            LOWER(COALESCE(stg.first_name, '') || ' ' || COALESCE(stg.middle_name, '') || ' ' || stg.last_name),
            LOWER(COALESCE(prod.first_name, '') || ' ' || COALESCE(prod.middle_name, '') || ' ' || prod.last_name)
        ) AS full_name_similarity,
        -- Match type indicators
        CASE WHEN stg.slug = prod.slug THEN 'exact_slug' 
             WHEN stg.ref_key = prod.ref_key AND stg.ref_key IS NOT NULL THEN 'exact_ref_key'
             WHEN stg.email = prod.email AND stg.email IS NOT NULL AND prod.email IS NOT NULL THEN 'exact_email'
             WHEN stg.phone = prod.phone AND stg.phone IS NOT NULL AND prod.phone IS NOT NULL AND stg.phone <> '' AND prod.phone <> '' THEN 'exact_phone'
             WHEN LOWER(stg.first_name) = LOWER(prod.first_name) 
                  AND LOWER(stg.last_name) = LOWER(prod.last_name) 
                  AND (stg.middle_name IS NULL OR prod.middle_name IS NULL 
                       OR LOWER(COALESCE(stg.middle_name, '')) = LOWER(COALESCE(prod.middle_name, ''))) 
             THEN 'exact_name'
             WHEN similarity(
                 LOWER(stg.first_name || ' ' || stg.last_name),
                 LOWER(prod.first_name || ' ' || prod.last_name)
             ) > 0.8 THEN 'high_similarity'
             WHEN similarity(
                 LOWER(stg.first_name || ' ' || stg.last_name),
                 LOWER(prod.first_name || ' ' || prod.last_name)
             ) > 0.6 THEN 'medium_similarity'
             ELSE 'low_similarity'
        END AS match_type
    FROM dbt_henry.stg_politicians stg
    CROSS JOIN politician prod
    WHERE 
        -- Exact matches
        stg.slug = prod.slug
        OR (stg.ref_key = prod.ref_key AND stg.ref_key IS NOT NULL)
        OR (stg.email = prod.email AND stg.email IS NOT NULL AND prod.email IS NOT NULL)
        OR (stg.phone = prod.phone AND stg.phone IS NOT NULL AND prod.phone IS NOT NULL AND stg.phone <> '' AND prod.phone <> '')
        OR (
            LOWER(stg.first_name) = LOWER(prod.first_name) 
            AND LOWER(stg.last_name) = LOWER(prod.last_name)
            AND (stg.middle_name IS NULL OR prod.middle_name IS NULL 
                 OR LOWER(COALESCE(stg.middle_name, '')) = LOWER(COALESCE(prod.middle_name, '')))
        )
        -- Similarity matches (threshold: 0.6 = 60% similar)
        OR similarity(
            LOWER(stg.first_name || ' ' || stg.last_name),
            LOWER(prod.first_name || ' ' || prod.last_name)
        ) > 0.6
)
SELECT *
FROM matches
ORDER BY 
    CASE match_type
        WHEN 'exact_slug' THEN 1
        WHEN 'exact_ref_key' THEN 2
        WHEN 'exact_email' THEN 3
        WHEN 'exact_phone' THEN 4
        WHEN 'exact_name' THEN 5
        WHEN 'high_similarity' THEN 6
        WHEN 'medium_similarity' THEN 7
        ELSE 8
    END,
    name_similarity DESC,
    staging_last_name,
    staging_first_name;

-- Option 3: Summary view showing match counts by type
SELECT 
    CASE 
        WHEN stg.slug = prod.slug THEN 'Exact Slug Match'
        WHEN stg.ref_key = prod.ref_key AND stg.ref_key IS NOT NULL THEN 'Exact Ref Key Match'
        WHEN stg.email = prod.email AND stg.email IS NOT NULL AND prod.email IS NOT NULL THEN 'Exact Email Match'
        WHEN stg.phone = prod.phone AND stg.phone IS NOT NULL AND prod.phone IS NOT NULL AND stg.phone <> '' AND prod.phone <> '' THEN 'Exact Phone Match'
        WHEN LOWER(stg.first_name) = LOWER(prod.first_name) 
             AND LOWER(stg.last_name) = LOWER(prod.last_name) THEN 'Exact Name Match'
        WHEN similarity(
            LOWER(stg.first_name || ' ' || stg.last_name),
            LOWER(prod.first_name || ' ' || prod.last_name)
        ) > 0.8 THEN 'High Similarity (>80%)'
        WHEN similarity(
            LOWER(stg.first_name || ' ' || stg.last_name),
            LOWER(prod.first_name || ' ' || prod.last_name)
        ) > 0.6 THEN 'Medium Similarity (60-80%)'
        ELSE 'Low Similarity (<60%)'
    END AS match_category,
    COUNT(*) AS match_count
FROM dbt_henry.stg_politicians stg
CROSS JOIN politician prod
WHERE 
    stg.slug = prod.slug
    OR (stg.ref_key = prod.ref_key AND stg.ref_key IS NOT NULL)
    OR (stg.email = prod.email AND stg.email IS NOT NULL AND prod.email IS NOT NULL)
    OR (stg.phone = prod.phone AND stg.phone IS NOT NULL AND prod.phone IS NOT NULL AND stg.phone <> '' AND prod.phone <> '')
    OR (
        LOWER(stg.first_name) = LOWER(prod.first_name) 
        AND LOWER(stg.last_name) = LOWER(prod.last_name)
    )
    OR similarity(
        LOWER(stg.first_name || ' ' || stg.last_name),
        LOWER(prod.first_name || ' ' || prod.last_name)
    ) > 0.6
GROUP BY match_category
ORDER BY 
    CASE match_category
        WHEN 'Exact Slug Match' THEN 1
        WHEN 'Exact Ref Key Match' THEN 2
        WHEN 'Exact Email Match' THEN 3
        WHEN 'Exact Phone Match' THEN 4
        WHEN 'Exact Name Match' THEN 5
        WHEN 'High Similarity (>80%)' THEN 6
        WHEN 'Medium Similarity (60-80%)' THEN 7
        ELSE 8
    END;

