-- Add down migration script here
ALTER TABLE question_issue_tags
DROP CONSTRAINT exclusion_question_issue_tags;
