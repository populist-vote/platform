BEGIN;

ALTER TABLE politician
DROP COLUMN ballotpedia_url;

COMMIT;
