-- Add up migration script here
-- Indexes for frequently filtered columns
CREATE INDEX IF NOT EXISTS idx_question_organization_id ON question (
    organization_id
);
CREATE INDEX IF NOT EXISTS idx_race_race_type ON race (race_type);
CREATE INDEX IF NOT EXISTS idx_office_political_scope ON office (
    political_scope
);
CREATE INDEX IF NOT EXISTS idx_office_state ON office (state);

-- Composite indexes
CREATE INDEX IF NOT EXISTS idx_race_race_type_state ON race (race_type, state);
CREATE INDEX IF NOT EXISTS idx_office_political_scope_state ON office (
    political_scope, state
);

-- Full-text search index on office
-- TODO - May want to add a materialized column office_document that is a concatenation of all text fields for indexing

-- Join optimization with indexes
CREATE INDEX IF NOT EXISTS idx_question_submission_question_id
ON question_submission (
    question_id
);
CREATE INDEX IF NOT EXISTS idx_candidate_guide_questions_question_id
ON candidate_guide_questions (
    question_id
);
CREATE INDEX IF NOT EXISTS idx_candidate_guide_questions_candidate_guide_id
ON candidate_guide_questions (
    candidate_guide_id
);
CREATE INDEX IF NOT EXISTS idx_candidate_guide_races_race_id
ON candidate_guide_races (
    race_id
);
CREATE INDEX IF NOT EXISTS idx_question_submission_candidate_id
ON question_submission (
    candidate_id
);
CREATE INDEX IF NOT EXISTS idx_race_candidates_race_candidate
ON race_candidates (
    race_id, candidate_id
);
