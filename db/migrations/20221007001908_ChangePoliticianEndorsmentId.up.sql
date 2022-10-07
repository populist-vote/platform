-- Add up migration script here
ALTER TABLE politician_politician_endorsements 
    DROP COLUMN id,
    DROP CONSTRAINT IF EXISTS politician_politician_endorsement_politician_endorsement_id_key,
    ADD UNIQUE (politician_id, politician_endorsement_id),
    ADD PRIMARY KEY (politician_id, politician_endorsement_id);

ALTER TABLE politician_organization_endorsements
  DROP COLUMN id,
  ADD UNIQUE (politician_id, organization_id),
  ADD PRIMARY KEY (politician_id, organization_id);