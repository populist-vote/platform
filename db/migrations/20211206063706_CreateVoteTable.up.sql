CREATE TABLE vote (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    populist_user_id uuid NOT NULL,
    votable_id uuid NOT NULL,
    votable_type VARCHAR(50) NOT NULL,
    direction INT NOT NULL CHECK (direction in (-1, 1)),
    created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    CONSTRAINT unique_person_per_votable UNIQUE(populist_user_id, votable_id)
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON vote
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();