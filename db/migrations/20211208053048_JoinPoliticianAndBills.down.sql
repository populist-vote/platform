-- Add down migration script here
ALTER TABLE bill
DROP COLUMN history;

ALTER TABLE legislation
RENAME COLUMN title TO name;

ALTER TABLE bill
DROP COLUMN votesmart_bill_id;

ALTER TABLE bill
DROP COLUMN bill_number;

ALTER TABLE bill
DROP COLUMN votesmart_bill_data;

ALTER TABLE bill
DROP CONSTRAINT IF EXISTS unique_votesmart_bill_id;

ALTER TABLE bill
DROP CONSTRAINT IF EXISTS unique_legiscan_bill_id;

DROP INDEX IF EXISTS bill_ids;