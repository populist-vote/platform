-- Add down migration script here
ALTER TABLE race
ALTER winner_ids TYPE uuid USING winner_ids[1];

ALTER TABLE race 
RENAME winner_ids TO winner_id;

ALTER TABLE race
ADD CONSTRAINT fk_politician FOREIGN KEY (winner_id) REFERENCES politician(id);

ALTER TABLE office
ADD COLUMN incumbent_id uuid REFERENCES politician(id);