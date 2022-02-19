-- Add up migration script here
CREATE TYPE political_scope AS ENUM ('local', 'state', 'federal');

CREATE TABLE office (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    office_type TEXT, /* useful for sorting */
    district TEXT,
    term_length INTEGER, /* in years */
    political_scope political_scope NOT NULL,
    encumbent_id uuid NOT NULL,
    municipality TEXT, /* for local offices */
    state state,
    created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    CONSTRAINT fk_politician FOREIGN KEY(encumbent_id) REFERENCES politician(id)
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON office
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();

CREATE TABLE race (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    office_position TEXT NOT NULL,
    office_id uuid NOT NULL,
    race_type TEXT NOT NULL DEFAULT 'primary',
    description TEXT,
    ballotpedia_link TEXT,
    early_voting_begins_date DATE,
    election_date DATE,
    official_website TEXT,
    state state,
    election_id uuid,
    created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    CONSTRAINT fk_election FOREIGN KEY(election_id) REFERENCES election(id),
    CONSTRAINT fk_office FOREIGN KEY(office_id) REFERENCES office(id)
);

ALTER TABLE politician
ADD COLUMN office_id uuid,
ADD COLUMN upcoming_race_id uuid,
ADD CONSTRAINT fk_office FOREIGN KEY(office_id) REFERENCES office(id) ON DELETE CASCADE,
ADD CONSTRAINT fk_race FOREIGN KEY(upcoming_race_id) REFERENCES race(id);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON race
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();