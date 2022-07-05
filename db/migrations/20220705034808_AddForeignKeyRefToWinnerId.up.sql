-- Add up migration script here
ALTER TABLE race 
ADD CONSTRAINT fk_politician FOREIGN KEY (winner_id) REFERENCES politician(id);