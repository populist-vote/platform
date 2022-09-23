-- Add down migration script here
ALTER TABLE politician
ADD CONSTRAINT fk_author_politician FOREIGN KEY (id) REFERENCES author(id) ON DELETE CASCADE;

CREATE OR REPLACE TRIGGER author_on_inserted_politician
  BEFORE INSERT ON politician
  FOR EACH ROW
  EXECUTE PROCEDURE create_author_from_politician();