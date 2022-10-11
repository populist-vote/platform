-- Add down migration script here
ALTER TABLE address
DROP COLUMN geom,
DROP COLUMN lat,
DROP COLUMN long;