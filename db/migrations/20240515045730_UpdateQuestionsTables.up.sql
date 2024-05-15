-- Add up migration script here
ALTER TABLE question_submission
ADD candidate_id uuid;

ALTER TABLE candidate_intake
DROP COLUMN candidate_id;

ALTER TABLE candidate_intake
ADD COLUMN name text;

ALTER TABLE candidate_intake
ADD COLUMN race_id uuid REFERENCES race (id) ON DELETE CASCADE;

ALTER TABLE candidate_intake
RENAME TO candidate_guide;


DROP TABLE candidate_intake_questions;

CREATE TABLE candidate_guide_questions (
    candidate_guide_id uuid NOT NULL,
    question_id uuid NOT NULL,
    FOREIGN KEY (candidate_guide_id) REFERENCES candidate_guide (
        id
    ) ON DELETE CASCADE,
    FOREIGN KEY (question_id) REFERENCES question (id) ON DELETE CASCADE
);

-- Create question_issue_tags table
CREATE TABLE question_issue_tags (
    question_id uuid NOT NULL,
    issue_tag_id uuid NOT NULL,
    created_at timestamptz DEFAULT NOW(),
    updated_at timestamptz DEFAULT NOW(),
    FOREIGN KEY (question_id) REFERENCES question (id) ON DELETE CASCADE,
    FOREIGN KEY (issue_tag_id) REFERENCES issue_tag (id) ON DELETE CASCADE
);

ALTER TABLE politician
ADD COLUMN intake_token text;
