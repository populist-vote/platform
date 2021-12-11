-- Add up migration script here

CREATE TABLE politician_bills (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    politician_id uuid NOT NULL,
    bill_id uuid NOT NULL,
    vote VARCHAR(1), -- TODO: perhaps make this a proper type with all possible vote types yea, nea, etc.
    created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    CONSTRAINT fk_politician FOREIGN KEY(politician_id) REFERENCES politician(id),
    CONSTRAINT fk_bill FOREIGN KEY(bill_id) REFERENCES bill(id)
);

ALTER TABLE legislation
RENAME COLUMN name TO title;

ALTER TABLE bill
ADD COLUMN votesmart_bill_id INT;

ALTER TABLE bill
ADD COLUMN bill_number TEXT NOT NULL;

ALTER TABLE bill
ADD COLUMN votesmart_bill_data JSONB NOT NULL DEFAULT '{}'::jsonb;

ALTER TABLE bill
ADD CONSTRAINT unique_votesmart_bill_id UNIQUE (votesmart_bill_id);

ALTER TABLE bill
ADD CONSTRAINT unique_legiscan_bill_id UNIQUE (legiscan_bill_id);

CREATE UNIQUE INDEX bill_ids ON bill (id, slug, bill_number, votesmart_bill_id, legiscan_bill_id);