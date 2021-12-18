-- Add up migration script here

ALTER TABLE bill
ADD COLUMN history JSONB NOT NULL DEFAULT '[]'::jsonb;

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