-- Add up migration script here
ALTER TABLE politician
ADD COLUMN assets JSONB NOT NULL DEFAULT '{}'::jsonb;

UPDATE politician SET assets['thumbnailImage160'] = to_jsonb(thumbnail_image_url) WHERE thumbnail_image_url IS NOT NULL;
UPDATE politician SET assets['thumbnailImage400'] = to_jsonb(FORMAT('https://populist-platform.s3.us-east-2.amazonaws.com/web-assets/politician-thumbnails/%s-400.jpg', slug)) WHERE thumbnail_image_url IS NOT NULL;