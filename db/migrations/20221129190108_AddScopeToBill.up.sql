-- Add up migration script here
ALTER TABLE bill
ADD COLUMN political_scope political_scope NOT NULL DEFAULT 'federal',
ADD COLUMN bill_type TEXT NOT NULL DEFAULT 'bill',
ADD COLUMN attributes JSONB NOT NULL DEFAULT '{}'::JSONB;

CREATE TABLE bill_public_votes (
  bill_id uuid REFERENCES bill(id) ON DELETE CASCADE,
  user_id uuid REFERENCES populist_user(id) ON DELETE CASCADE,
  position argument_position NOT NULL,
  attributes JSONB NOT NULL DEFAULT '{}'::JSONB
);

CREATE TABLE ballot_measure_public_votes (
  ballot_measure_id uuid REFERENCES bill(id) ON DELETE CASCADE,
  user_id uuid REFERENCES populist_user(id) ON DELETE CASCADE,
  position argument_position NOT NULL,
  attributes JSONB NOT NULL DEFAULT '{}'::JSONB
);
