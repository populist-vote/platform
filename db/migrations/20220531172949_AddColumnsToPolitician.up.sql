-- Add up migration script here
ALTER TABLE politician
DROP COLUMN description,
ADD COLUMN suffix TEXT,
ADD COLUMN biography TEXT,
ADD COLUMN biography_source TEXT,
ADD COLUMN campaign_website_url TEXT,
ADD COLUMN tiktok_url TEXT,
ADD COLUMN linkedin_url TEXT,
ADD COLUMN youtube_url TEXT,
ADD COLUMN email TEXT; 