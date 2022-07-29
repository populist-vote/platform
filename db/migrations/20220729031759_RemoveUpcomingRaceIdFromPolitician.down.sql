-- Add down migration script here
ALTER TABLE politician
ADD COLUMN upcoming_race_id uuid,
ADD CONSTRAINT fk_race FOREIGN KEY(upcoming_race_id) REFERENCES race(id);