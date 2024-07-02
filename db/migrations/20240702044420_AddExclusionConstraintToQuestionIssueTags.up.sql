-- Add up migration script here
ALTER TABLE question_issue_tags
ADD CONSTRAINT exclusion_question_issue_tags UNIQUE (question_id, issue_tag_id);
