-- Add down migration script here

CREATE TYPE political_party_new AS ENUM ('democratic', 'republican', 'libertarian', 'green', 'constitution', 'unknown', 'independent', 'freedom', 'unity', 'approval_voting', 'unaffiliated', 'democratic_farmer_labor', 'grassroots_legalize_cannabis', 'legal_marijuana_now', 'socialist_workers_party', 'socialist_workers');
ALTER TABLE politician ALTER COLUMN party TYPE political_party_new USING party::text::political_party_new;
ALTER TABLE race ALTER COLUMN party TYPE political_party_new USING party::text::political_party_new;
ALTER TABLE user_profile ALTER COLUMN party TYPE political_party_new USING party::text::political_party_new;
DROP TYPE political_party;
ALTER TYPE political_party_new RENAME TO political_party;