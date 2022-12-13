-- Add up migration script here
CREATE TYPE bill_status AS ENUM ('introduced', 'in_consideration', 'became_law', 'failed', 'vetoed', 'unknown');
CREATE TYPE ballot_measure_status AS ENUM ('introduced', 'in_consideration', 'proposed', 'gathering_signatures', 'on_the_ballot', 'became_law', 'failed', 'unknown');

ALTER TABLE bill ADD COLUMN status bill_status NOT NULL DEFAULT 'unknown';
ALTER TABLE ballot_measure ADD COLUMN status ballot_measure_status NOT NULL DEFAULT 'unknown';

UPDATE bill SET status = CASE legislation_status
  WHEN 'introduced' THEN 'introduced'::bill_status
  WHEN 'passed_house' THEN 'in_consideration'::bill_status
  WHEN 'passed_senate' THEN 'in_consideration'::bill_status
  WHEN 'failed_house' THEN 'failed'::bill_status
  WHEN 'failed_senate' THEN 'failed'::bill_status
  WHEN 'resolving_differences' THEN 'in_consideration'::bill_status
  WHEN 'sent_to_executive' THEN 'in_consideration'::bill_status
  WHEN 'became_law' THEN 'became_law'::bill_status
  WHEN 'vetoed' THEN 'vetoed'::bill_status
  WHEN 'became_law' THEN 'became_law'::bill_status
  ELSE 'unknown'
END;

-- NO ballot_measures in our data currently, no need to update them

ALTER TABLE legislation DROP COLUMN legislation_status;
DROP TYPE legislation_status;