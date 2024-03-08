-- Add up migration script here
ALTER TABLE session
ADD COLUMN legiscan_dataset_hash TEXT;
