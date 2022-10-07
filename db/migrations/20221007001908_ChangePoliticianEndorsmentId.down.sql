-- Add down migration script here
ALTER TABLE politician_politician_endorsements 
    DROP CONSTRAINT politician_politician_endorse_politician_id_politician_endo_key,
    DROP CONSTRAINT politician_politician_endorsements_pkey,
    ADD COLUMN IF NOT EXISTS id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY;

ALTER TABLE politician_organization_endorsements
    DROP CONSTRAINT politician_organization_endorsements_pkey,
    ADD COLUMN IF NOT EXISTS id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY;