-- Add up migration script here
CREATE TABLE bill_sponsors
(
  PRIMARY KEY (bill_id, politician_id),
  bill_id uuid NOT NULL,
  politician_id uuid NOT NULL,
  created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
  updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
  CONSTRAINT fk_bill FOREIGN KEY(bill_id) REFERENCES bill(id),
  CONSTRAINT fk_politician FOREIGN KEY(politician_id) REFERENCES politician(id)
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON bill_sponsors
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();