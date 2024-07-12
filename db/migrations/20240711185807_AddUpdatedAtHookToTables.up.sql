-- Add up migration script here

CREATE TRIGGER set_updated_at
BEFORE UPDATE
ON candidate_guide
FOR EACH ROW
EXECUTE PROCEDURE set_updated_at();
