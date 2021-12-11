-- Add down migration script here
DROP TABLE politician_bills;

ALTER TABLE legislation
RENAME COLUMN title TO name;

ALTER TABLE bill
DROP COLUMN votesmart_bill_id;

ALTER TABLE bill
DROP COLUMN bill_number;

ALTER TABLE bill
DROP COLUMN votesmart_bill_data;

-- ALTER TABLE bill
-- DROP CONSTRAINT unique_votesmart_bill_id;

ALTER TABLE bill
DROP CONSTRAINT unique_legiscan_bill_id;

-- DROP INDEX bill_ids;