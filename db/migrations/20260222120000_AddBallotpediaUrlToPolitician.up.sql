BEGIN;

ALTER TABLE politician
ADD COLUMN ballotpedia_url TEXT;

COMMIT;
