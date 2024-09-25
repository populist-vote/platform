-- Add up migration script here
CREATE OR REPLACE FUNCTION ensure_user_profile_exists()
RETURNS TRIGGER AS $$
BEGIN
    -- Insert a user_profile if it doesn't exist for the new user
    INSERT INTO user_profile (user_id, updated_at)
    VALUES (NEW.id, NOW())
    ON CONFLICT (user_id) DO NOTHING; -- Ensure that if profile already exists, no action is taken

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Step 2: Create a trigger that calls the function whenever a 
-- populist_user is inserted
CREATE TRIGGER create_user_profile
AFTER INSERT ON populist_user
FOR EACH ROW
EXECUTE FUNCTION ensure_user_profile_exists();

-- Step 3: Optional - Add a foreign key constraint to ensure that 
-- every user_profile  has a valid populist_user
ALTER TABLE user_profile
ADD CONSTRAINT fk_user_profile_user
FOREIGN KEY (user_id)
REFERENCES populist_user (id)
ON UPDATE CASCADE
ON DELETE CASCADE;
