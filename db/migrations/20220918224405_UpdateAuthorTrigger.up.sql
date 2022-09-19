-- Add up migration script here
ALTER TABLE politician
DROP CONSTRAINT fk_author_politician;

CREATE OR REPLACE TRIGGER author_on_inserted_politician
  AFTER INSERT ON politician
  FOR EACH ROW
  EXECUTE PROCEDURE create_author_from_politician();