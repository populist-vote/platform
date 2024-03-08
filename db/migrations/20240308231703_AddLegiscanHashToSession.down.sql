-- Add down migration script here
ALTER TABLE session DROP COLUMN legiscan_dataset_hash;
