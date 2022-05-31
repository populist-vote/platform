-- Add down migration script here
ALTER TABLE politician
ADD COLUMN description TEXT,
DROP COLUMN suffix,
DROP COLUMN biography,
DROP COLUMN biography_source,
DROP COLUMN campaign_website_url,
DROP COLUMN tiktok_url,
DROP COLUMN linkedin_url,
DROP COLUMN youtube_url,
DROP COLUMN email;