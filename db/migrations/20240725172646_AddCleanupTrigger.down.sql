-- Add down migration script here

-- Down migration for the trigger on candidate_guide
DROP TRIGGER IF EXISTS trg_delete_embeds_on_candidate_guide_delete ON candidate_guide;
DROP FUNCTION IF EXISTS delete_embeds_on_candidate_guide_delete ();

-- Down migration for the trigger on candidate_guide_races
DROP TRIGGER IF EXISTS trg_delete_embeds_on_candidate_guide_race_delete ON candidate_guide_races;
DROP FUNCTION IF EXISTS delete_embeds_on_candidate_guide_race_delete ();

-- Down migration for the trigger on race
DROP TRIGGER IF EXISTS trg_delete_embeds_on_race_delete ON race;
DROP FUNCTION IF EXISTS delete_embeds_on_race_delete ();

ALTER TABLE candidate_guide_races
DROP CONSTRAINT IF EXISTS unique_candidate_guide_race;
