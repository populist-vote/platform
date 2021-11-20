-- Add up migration script here
CREATE TYPE argument_position AS ENUM ('support', 'neutral', 'oppose');
CREATE TYPE author_type AS ENUM ('politician', 'organization');

CREATE TABLE author (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    author_type author_type NOT NULL
);

ALTER TABLE politician
ADD CONSTRAINT fk_author_politician FOREIGN KEY (id) REFERENCES author(id);

ALTER TABLE organization
ADD CONSTRAINT fk_author_organization FOREIGN KEY (id) references author(id);

CREATE TABLE argument (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    author_id uuid NOT NULL,
    title TEXT NOT NULL,
    body TEXT,
    position argument_position NOT NULL,
    created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    CONSTRAINT fk_author FOREIGN KEY(author_id) REFERENCES author(id)
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON argument
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();

CREATE TABLE bill_arguments (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    bill_id uuid NOT NULL,
    argument_id uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    CONSTRAINT fk_bill FOREIGN KEY(bill_id) REFERENCES bill(id),
    CONSTRAINT fk_argument FOREIGN KEY(argument_id) REFERENCES argument(id)
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON bill_arguments
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();

CREATE TABLE ballot_measure_arguments (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    ballot_measure_id uuid NOT NULL,
    argument_id uuid NOT NULL,
    created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    CONSTRAINT fk_ballot_measure FOREIGN KEY(ballot_measure_id) REFERENCES ballot_measure(id),
    CONSTRAINT fk_argument FOREIGN KEY(argument_id) REFERENCES argument(id)
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON ballot_measure_arguments
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();
