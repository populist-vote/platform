BEGIN;

ALTER TABLE ballot_measure
ADD CONSTRAINT ballot_measure_ballot_measure_code_key UNIQUE (ballot_measure_code);

COMMIT;
