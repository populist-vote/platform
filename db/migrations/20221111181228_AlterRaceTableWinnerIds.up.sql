-- Add up migration script here

ALTER TABLE race 
DROP CONSTRAINT IF EXISTS fk_politician,
ALTER winner_id TYPE uuid[] USING array[winner_id];

ALTER TABLE race
RENAME winner_id TO winner_ids;

ALTER TABLE office
DROP column incumbent_id;