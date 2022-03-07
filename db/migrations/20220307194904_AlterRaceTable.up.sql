-- Add up migration script here
CREATE TYPE race_type AS ENUM ('primary', 'general');

ALTER TABLE race
DROP COLUMN race_type, 
ADD COLUMN race_type race_type NOT NULL DEFAULT 'general',
ADD COLUMN party political_party;

ALTER TABLE office
ADD COLUMN description TEXT;

ALTER TABLE office
RENAME encumbent_id TO incumbent_id;