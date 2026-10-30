BEGIN;

ALTER TABLE ballot_measure
DROP CONSTRAINT ballot_measure_ballot_measure_code_key;

COMMIT;
