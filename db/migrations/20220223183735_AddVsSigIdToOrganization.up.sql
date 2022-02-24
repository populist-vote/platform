-- Add up migration script here
ALTER TABLE organization
ADD COLUMN votesmart_sig_id INT NULL,
ADD COLUMN headquarters_address_id uuid,
ADD CONSTRAINT fk_headquarters_address_id FOREIGN KEY(headquarters_address_id) REFERENCES address(id) ON DELETE SET NULL;

CREATE UNIQUE INDEX org_votesmart_sid_id ON organization (votesmart_sig_id);