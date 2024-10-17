ALTER TABLE politician_politician_endorsements
DROP COLUMN start_date,
DROP COLUMN end_date,
DROP CONSTRAINT IF EXISTS unique_politician_endorsement_start;

ALTER TABLE politician_politician_endorsements
ADD CONSTRAINT politician_politician_endorsements_pkey PRIMARY KEY (
    politician_id, politician_endorsement_id
),
ADD CONSTRAINT politician_politician_endorse_politician_id_politician_endo_key
UNIQUE (
    politician_id, politician_endorsement_id
);

ALTER TABLE politician_organization_endorsements
DROP COLUMN start_date,
DROP COLUMN end_date,
DROP CONSTRAINT IF EXISTS unique_politician_org_endorsement_start;

ALTER TABLE politician_organization_endorsements
ADD CONSTRAINT politician_organization_endorsements_pkey PRIMARY KEY (
    politician_id, organization_id
),
ADD CONSTRAINT politician_organization_endor_politician_id_organization_id_key
UNIQUE (
    politician_id, organization_id
);
