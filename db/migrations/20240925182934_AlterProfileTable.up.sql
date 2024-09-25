-- Add up migration script here
ALTER TABLE user_profile
DROP CONSTRAINT IF EXISTS fk_user_profile_user,
DROP CONSTRAINT IF EXISTS fk_address,
ADD CONSTRAINT fk_address FOREIGN KEY (address_id) REFERENCES address (
    id
) ON DELETE SET NULL;
