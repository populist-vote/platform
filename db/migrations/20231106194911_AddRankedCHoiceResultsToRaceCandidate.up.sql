-- Add up migration script here
ALTER TABLE race_candidates ADD COLUMN IF NOT EXISTS ranked_choice_results JSONB;
