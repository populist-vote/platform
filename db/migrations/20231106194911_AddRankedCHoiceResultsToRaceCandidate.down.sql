-- Add down migration script here
ALTER TABLE race_candidates DROP COLUMN IF EXISTS ranked_choice_results;
