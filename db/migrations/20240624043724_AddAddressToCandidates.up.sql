-- Add up migration script here
ALTER TABLE politician
ADD COLUMN residence_address_id UUID REFERENCES address (id),
ADD COLUMN campaign_address_id UUID REFERENCES address (id);
