-- Add up migration script here
ALTER TABLE race
ADD COLUMN is_special_election boolean NOT NULL DEFAULT(false),
ADD COLUMN num_elect INT;