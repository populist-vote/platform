-- Add up migration script here
CREATE OR REPLACE FUNCTION create_author_from_politician()
RETURNS TRIGGER AS $$
BEGIN
  INSERT INTO author(author_type, id)
  VALUES ('politician', NEW.id);
  RETURN NEW;
END;
$$
LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER author_on_inserted_politician
  BEFORE INSERT ON politician
  FOR EACH ROW
  EXECUTE PROCEDURE create_author_from_politician();
