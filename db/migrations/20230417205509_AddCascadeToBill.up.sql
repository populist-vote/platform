-- Add up migration script here
ALTER TABLE bill_sponsors
DROP CONSTRAINT fk_bill,
ADD CONSTRAINT fk_bill
	FOREIGN KEY (bill_id) 
	REFERENCES bill(id)
	ON DELETE CASCADE;

ALTER TABLE bill_issue_tags
DROP CONSTRAINT fk_bill,
ADD CONSTRAINT fk_bill
	FOREIGN KEY (bill_id) 
	REFERENCES bill(id)
	ON DELETE CASCADE;