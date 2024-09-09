-- Add up migration script here
CREATE OR REPLACE FUNCTION notify_new_embed_origin()
RETURNS TRIGGER AS $$
BEGIN
  PERFORM pg_notify('new_embed_origin', NEW.url);
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER embed_origin_insert
AFTER INSERT ON embed_origin
FOR EACH ROW
EXECUTE FUNCTION notify_new_embed_origin();
