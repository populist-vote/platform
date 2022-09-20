-- Add down migration script here
CREATE OR REPLACE TRIGGER author_on_inserted_politician
  AFTER INSERT ON politician
  FOR EACH ROW
  EXECUTE PROCEDURE create_author_from_politician();