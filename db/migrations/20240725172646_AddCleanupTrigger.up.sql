-- Add up migration script here
CREATE OR REPLACE FUNCTION delete_embeds_on_candidate_guide_delete()
RETURNS TRIGGER AS $$
DECLARE
    candidate_guide_id UUID;
BEGIN
    -- Use the id of the deleted candidate_guide directly
    candidate_guide_id := OLD.id;
    
    -- Delete embeds with the candidate_guide_id from attributes JSONB column
    DELETE FROM embed WHERE (attributes->>'candidateGuideId')::UUID = candidate_guide_id;
    
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_delete_embeds_on_candidate_guide_delete
AFTER DELETE ON candidate_guide
FOR EACH ROW
EXECUTE FUNCTION delete_embeds_on_candidate_guide_delete();

CREATE OR REPLACE FUNCTION delete_embeds_on_candidate_guide_race_delete()
RETURNS TRIGGER AS $$
DECLARE
    candidate_guide_id UUID;
    race_id UUID;
BEGIN
    -- Extract candidate_guide_id and race_id from attributes JSONB column
    candidate_guide_id := OLD.candidate_guide_id;
    race_id := OLD.race_id;
    
    -- Delete embeds with the extracted candidate_guide_id and race_id
    DELETE FROM embed WHERE (attributes->>'candidateGuideId')::UUID = candidate_guide_id AND 
        (attributes->>'raceId')::UUID = race_id;
    
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_delete_embeds_on_candidate_guide_race_delete
AFTER DELETE ON candidate_guide_races
FOR EACH ROW
EXECUTE FUNCTION delete_embeds_on_candidate_guide_race_delete();

CREATE OR REPLACE FUNCTION delete_embeds_on_race_delete()
RETURNS TRIGGER AS $$
DECLARE
    race_id UUID;
BEGIN
    -- Delete embeds with the deleted race_id
    race_id := OLD.id;
    DELETE FROM embed WHERE (attributes->>'race_id')::UUID = race_id;
    
    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_delete_embeds_on_race_delete
AFTER DELETE ON race
FOR EACH ROW
EXECUTE FUNCTION delete_embeds_on_race_delete();

-- Delete duplicates from candidate_guide_races
WITH ranked_rows AS (
    SELECT
        candidate_guide_id,
        race_id,
        row_number()
            OVER (
                PARTITION BY candidate_guide_id, race_id ORDER BY (SELECT NULL)
            )
        AS rnum
    FROM
        candidate_guide_races
)

DELETE FROM candidate_guide_races
USING ranked_rows
WHERE
    candidate_guide_races.candidate_guide_id = ranked_rows.candidate_guide_id
    AND candidate_guide_races.race_id = ranked_rows.race_id
    AND ranked_rows.rnum > 1;

-- Add unique constraint to candidate_guide_races
ALTER TABLE candidate_guide_races
ADD CONSTRAINT unique_candidate_guide_race
UNIQUE (candidate_guide_id, race_id);
