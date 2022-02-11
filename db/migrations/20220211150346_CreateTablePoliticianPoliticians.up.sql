-- Add up migration script here
ALTER TABLE IF EXISTS politician_endorsements
RENAME TO politician_organization_endorsements;

CREATE TABLE politician_politician_endorsements
(
  id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  politician_id uuid NOT NULL,
  politician_endorsement_id uuid NOT NULL,
  created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
  updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
  CONSTRAINT fk_politician FOREIGN KEY(politician_id) REFERENCES politician(id),
  CONSTRAINT fk_organization FOREIGN KEY(politician_endorsement_id) REFERENCES politician(id)
);