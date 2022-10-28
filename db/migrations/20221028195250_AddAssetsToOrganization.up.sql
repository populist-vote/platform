-- Add up migration script here
ALTER TABLE "organization" ADD COLUMN "assets" jsonb NOT NULL DEFAULT '{}'::jsonb;