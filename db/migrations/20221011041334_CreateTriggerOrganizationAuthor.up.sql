-- Add up migration script here
CREATE OR REPLACE FUNCTION create_author_from_organization()
RETURNS TRIGGER AS $$
BEGIN
  INSERT INTO author(author_type, id)
  VALUES ('organization', NEW.id)
  ON CONFLICT DO NOTHING;
  RETURN NEW;
END;
$$
LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER author_on_inserted_organization
  BEFORE INSERT ON organization
  FOR EACH ROW
  EXECUTE PROCEDURE create_author_from_organization();