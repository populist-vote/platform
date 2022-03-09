-- Add down migration script here

ALTER TABLE race
DROP COLUMN race_type, 
ADD COLUMN race_type TEXT NOT NULL DEFAULT 'primary',
DROP COLUMN party;

DROP TYPE race_type;

ALTER TABLE office
DROP COLUMN description;

ALTER TABLE office
RENAME incumbent_id TO encumbent_id;

ALTER TABLE politician
RENAME party TO office_party;