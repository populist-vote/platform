-- Add down migration script here
-- Indexes for frequently filtered columns
DROP INDEX IF EXISTS idx_question_organization_id;
DROP INDEX IF EXISTS idx_race_race_type;
DROP INDEX IF EXISTS idx_office_political_scope;
DROP INDEX IF EXISTS idx_office_state;

-- Composite indexes
DROP INDEX IF EXISTS idx_race_race_type_state;
DROP INDEX IF EXISTS idx_office_political_scope_state;



-- Join optimization with indexes
DROP INDEX IF EXISTS idx_question_submission_question_id;
DROP INDEX IF EXISTS idx_candidate_guide_questions_question_id;
DROP INDEX IF EXISTS idx_candidate_guide_questions_candidate_guide_id;
DROP INDEX IF EXISTS idx_candidate_guide_races_race_id;
DROP INDEX IF EXISTS idx_question_submission_candidate_id;
DROP INDEX IF EXISTS idx_race_candidates_race_candidate;
