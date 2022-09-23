-- Add up migration script here
ALTER TABLE office
ADD COLUMN priority INTEGER;

UPDATE office SET priority = 1 WHERE title LIKE 'U.S. President';
UPDATE office SET priority = 2 WHERE title LIKE 'U.S. Vice President';
UPDATE office SET priority = 3 WHERE title LIKE 'U.S. Senator';
UPDATE office SET priority = 4 WHERE title LIKE 'U.S. Representative';
UPDATE office SET priority = 5 WHERE title LIKE 'Governor';
UPDATE office SET priority = 6 WHERE title LIKE 'Lieutenant Governor';
UPDATE office SET priority = 7 WHERE title LIKE 'Secretary of State';
UPDATE office SET priority = 8 WHERE title LIKE 'Attorney General';
UPDATE office SET priority = 9 WHERE title LIKE 'State %Auditor%';
UPDATE office SET priority = 10 WHERE title LIKE 'State Senator';
UPDATE office SET priority = 11 WHERE title LIKE 'State Representative';
UPDATE office SET priority = 12 WHERE title LIKE 'Mayor';
UPDATE office SET priority = 13 WHERE title LIKE 'City Council';
UPDATE office SET priority = 14 WHERE title LIKE '%County%';
UPDATE office SET priority = 15 WHERE title LIKE '%Supreme Court%';
UPDATE office SET priority = 16 WHERE title LIKE '%Court of Appeals%';
UPDATE office SET priority = 17 WHERE title LIKE '%Judge%' AND election_scope = 'district';


