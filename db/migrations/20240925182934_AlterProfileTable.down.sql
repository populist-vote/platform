-- Add down migration script here
ALTER TABLE user_profile
DROP CONSTRAINT fk_user,
DROP CONSTRAINT fk_address,
ADD CONSTRAINT fk_user FOREIGN KEY (user_id) REFERENCES populist_user (id)
ON DELETE CASCADE,
ADD CONSTRAINT fk_address FOREIGN KEY (address_id) REFERENCES address (id)
ON DELETE CASCADE;
