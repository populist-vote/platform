-- Add down migration script here
ALTER TYPE political_party RENAME TO political_party_old;
CREATE TYPE political_party AS ENUM ('democratic','republican','libertarian','green','constitution','unknown','independent','freedom','unity','approval_voting','unaffiliated','democratic_farmer_labor','grassroots_legalize_cannabis','legal_marijuana_now','socialist_workers', 'socialist_workers_party', 'colorado_center','american_constitution');
ALTER TABLE politician ALTER COLUMN party TYPE political_party USING party::text::political_party;
ALTER TABLE race ALTER COLUMN party TYPE political_party USING party::text::political_party;
ALTER TABLE user_profile ALTER COLUMN party TYPE political_party USING party::text::political_party;
DROP TYPE political_party_old;
