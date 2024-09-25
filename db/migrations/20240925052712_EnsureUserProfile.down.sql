-- Add down migration script here
-- Drop the foreign key constraint
ALTER TABLE user_profile
DROP CONSTRAINT IF EXISTS fk_user_profile_user;

-- Drop the trigger
DROP TRIGGER IF EXISTS create_user_profile ON populist_user;

-- Drop the function
DROP FUNCTION IF EXISTS ensure_user_profile_exists;
