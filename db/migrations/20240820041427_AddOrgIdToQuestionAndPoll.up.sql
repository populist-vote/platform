ALTER TABLE question ADD COLUMN organization_id UUID REFERENCES organization (
    id
);
ALTER TABLE poll ADD COLUMN organization_id UUID REFERENCES organization (id);

UPDATE question
SET organization_id = embed.organization_id
FROM embed
WHERE question.id = (embed.attributes ->> 'questionId')::UUID;

UPDATE question
SET organization_id = candidate_guide.organization_id
FROM candidate_guide_questions
INNER JOIN
    candidate_guide
    ON candidate_guide_questions.candidate_guide_id = candidate_guide.id
WHERE question.id = candidate_guide_questions.question_id;

UPDATE poll
SET organization_id = embed.organization_id
FROM embed
WHERE poll.id = (embed.attributes ->> 'pollId')::UUID;

ALTER TABLE question ALTER COLUMN organization_id SET NOT NULL;
ALTER TABLE poll ALTER COLUMN organization_id SET NOT NULL;
