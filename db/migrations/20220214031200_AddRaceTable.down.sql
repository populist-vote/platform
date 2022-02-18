ALTER TABLE politician
DROP COLUMN office_id,
DROP COLUMN upcoming_race_id,
DROP CONSTRAINT IF EXISTS fk_office,
DROP CONSTRAINT IF EXISTS fk_race;

DROP TABLE race;
DROP TABLE office;
DROP TYPE political_scope;
