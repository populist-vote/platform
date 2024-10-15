-- Add up migration script here
ALTER TABLE politician_politician_endorsements
ADD COLUMN election_id UUID REFERENCES election (id);

ALTER TABLE politician_organization_endorsements
ADD COLUMN election_id UUID REFERENCES election (id);
