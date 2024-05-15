-- Add down migration script here
ALTER TABLE question_submission
DROP COLUMN candidate_id;

ALTER TABLE candidate_guide
ADD COLUMN candidate_id uuid;

ALTER TABLE candidate_guide
DROP COLUMN name;

ALTER TABLE candidate_guide
DROP COLUMN race_id;

ALTER TABLE candidate_guide
RENAME TO candidate_intake;

DROP TABLE candidate_guide_questions;

CREATE TABLE candidate_intake_questions (
    candidate_intake_id uuid NOT NULL,
    question_id uuid NOT NULL,
    created_at timestamptz DEFAULT now(),
    updated_at timestamptz DEFAULT now(),
    FOREIGN KEY (candidate_intake_id) REFERENCES candidate_intake (
        id
    ) ON DELETE CASCADE,
    FOREIGN KEY (question_id) REFERENCES question (id) ON DELETE CASCADE
);

-- Drop question_issue_tags table
DROP TABLE IF EXISTS question_issue_tags;

ALTER TABLE politician
DROP COLUMN intake_token;
