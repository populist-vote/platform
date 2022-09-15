-- Add down migration script here
CREATE TYPE political_party_new AS ENUM ('democratic', 'republican', 'libertarian', 'freedom', 'unity', 'green', 'constitution', 'independent', 'approval_voting', 'unaffiliated', 'unknown');
ALTER TABLE politician ALTER COLUMN party TYPE political_party_new USING party::text::political_party_new;
ALTER TABLE race ALTER COLUMN party TYPE political_party_new USING party::text::political_party_new;
ALTER TABLE user_profile ALTER COLUMN party TYPE political_party_new USING party::text::political_party_new;
DROP TYPE political_party;
ALTER TYPE political_party_new RENAME TO political_party;