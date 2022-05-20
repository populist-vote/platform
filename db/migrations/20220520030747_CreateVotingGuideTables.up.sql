-- Add up migration script here
CREATE TABLE voting_guide (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    user_id uuid NOT NULL,
    title TEXT,
    description TEXT,
    created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    CONSTRAINT fk_user FOREIGN KEY(user_id) REFERENCES populist_user(id)
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON voting_guide
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();

CREATE TABLE voting_guide_candidates (
    voting_guide_id uuid NOT NULL,
    candidate_id uuid NOT NULL,
    is_endorsement boolean NOT NULL DEFAULT false,
    note TEXT,
    created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    CONSTRAINT fk_voting_guide FOREIGN KEY(voting_guide_id) REFERENCES voting_guide(id),
    CONSTRAINT fk_candidate FOREIGN KEY(candidate_id) REFERENCES politician(id),
    UNIQUE (voting_guide_id, candidate_id)
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON voting_guide_candidates
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();