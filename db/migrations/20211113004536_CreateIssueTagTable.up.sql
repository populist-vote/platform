-- Add up migration script here
CREATE TABLE issue_tag (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    name  TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
);

CREATE TABLE politician_issue_tags (
  id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  politician_id uuid NOT NULL,
  issue_tag_id uuid NOT NULL,
  CONSTRAINT fk_politician FOREIGN KEY(politician_id) REFERENCES politician(id),
  CONSTRAINT fk_issue_tag FOREIGN KEY(issue_tag_id) REFERENCES issue_tag(id)
);

CREATE TABLE organization_issue_tags (
  id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  organization_id uuid NOT NULL,
  issue_tag_id uuid NOT NULL,
  CONSTRAINT fk_organization FOREIGN KEY(organization_id) REFERENCES organization(id),
  CONSTRAINT fk_issue_tag FOREIGN KEY(issue_tag_id) REFERENCES issue_tag(id)
);

CREATE TABLE bill_issue_tags (
  id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  bill_id uuid NOT NULL,
  issue_tag_id uuid NOT NULL,
  CONSTRAINT fk_bill FOREIGN KEY(bill_id) REFERENCES bill(id),
  CONSTRAINT fk_issue_tag FOREIGN KEY(issue_tag_id) REFERENCES issue_tag(id)
);

CREATE TABLE ballot_measure_issue_tags (
  id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  ballot_measure_id uuid NOT NULL,
  issue_tag_id uuid NOT NULL,
  CONSTRAINT fk_ballot_measure FOREIGN KEY(ballot_measure_id) REFERENCES ballot_measure(id),
  CONSTRAINT fk_issue_tag FOREIGN KEY(issue_tag_id) REFERENCES issue_tag(id)
);