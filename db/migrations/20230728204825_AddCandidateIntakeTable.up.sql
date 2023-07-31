-- Add up migration script here
CREATE TABLE candidate_intake (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    candidate_id UUID NOT NULL REFERENCES politician(id),
    organization_id UUID NOT NULL REFERENCES organization(id),
    populist_url TEXT,
    created_by UUID NOT NULL REFERENCES populist_user(id),
    created_at timestamptz  NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at timestamptz  NOT NULL DEFAULT CURRENT_TIMESTAMP
);
-- Bridge table to connect questions to intakes
CREATE TABLE candidate_intake_questions (
    candidate_intake_id UUID NOT NULL REFERENCES candidate_intake(id)
)