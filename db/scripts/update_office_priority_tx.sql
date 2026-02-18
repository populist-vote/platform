-- Set office.priority for Texas offices by matching office name (or title) to the scheme in
-- platform/scrapers/src/generators/tx/office.rs office_priority().
-- Run from platform/db with DATABASE_URL set, or: psql $DATABASE_URL -f scripts/update_office_priority_tx.sql

-- Only Texas offices (state = 'TX'). Match on title.
-- Federal
UPDATE office SET priority = 1 WHERE state = 'TX' AND title = 'U.S. President';
UPDATE office SET priority = 2 WHERE state = 'TX' AND title = 'U.S. Vice President';
UPDATE office SET priority = 3 WHERE state = 'TX' AND title = 'U.S. Senator';
UPDATE office SET priority = 4 WHERE state = 'TX' AND title = 'U.S. Representative';

-- State executive
UPDATE office SET priority = 5 WHERE state = 'TX' AND title = 'Governor';
UPDATE office SET priority = 6 WHERE state = 'TX' AND title = 'Lieutenant Governor';
UPDATE office SET priority = 7 WHERE state = 'TX' AND title = 'Secretary of State';
UPDATE office SET priority = 8 WHERE state = 'TX' AND title = 'Attorney General';
UPDATE office SET priority = 9 WHERE state = 'TX' AND title = 'Comptroller of Public Accounts';
UPDATE office SET priority = 10 WHERE state = 'TX' AND title = 'Commissioner of the General Land Office';
UPDATE office SET priority = 11 WHERE state = 'TX' AND title = 'Commissioner of Agriculture';
UPDATE office SET priority = 12 WHERE state = 'TX' AND title = 'Railroad Commissioner';

-- State courts (Supreme / CCA)
UPDATE office SET priority = 15 WHERE state = 'TX' AND title = 'Chief Justice - Supreme Court';
UPDATE office SET priority = 16 WHERE state = 'TX' AND title = 'Justice - Supreme Court';
UPDATE office SET priority = 17 WHERE state = 'TX' AND title = 'Judge - Court of Criminal Appeals';

-- State legislature / board
UPDATE office SET priority = 18 WHERE state = 'TX' AND title = 'State Board of Education Member';
UPDATE office SET priority = 19 WHERE state = 'TX' AND title = 'State Senator';
UPDATE office SET priority = 20 WHERE state = 'TX' AND title = 'State Representative';

-- Court of Appeals: district 15 → 21 (Chief) / 22 (Justice); else 23 / 24
UPDATE office SET priority = 21
WHERE state = 'TX' AND title = 'Chief Justice - Court of Appeals' AND district = '15';
UPDATE office SET priority = 23
WHERE state = 'TX' AND title = 'Chief Justice - Court of Appeals' AND district != '15';
UPDATE office SET priority = 22
WHERE state = 'TX' AND title = 'Justice - Court of Appeals' AND district = '15';
UPDATE office SET priority = 24
WHERE state = 'TX' AND title = 'Justice - Court of Appeals' AND district != '15';

-- District / county judicial and officials
UPDATE office SET priority = 25 WHERE state = 'TX' AND title = 'District Judge';
UPDATE office SET priority = 26 WHERE state = 'TX' AND title = 'District Attorney';
UPDATE office SET priority = 27 WHERE state = 'TX' AND title = 'Criminal District Judge';
UPDATE office SET priority = 28 WHERE state = 'TX' AND title = 'Criminal District Attorney';
UPDATE office SET priority = 30 WHERE state = 'TX' AND title = 'County Judge';
UPDATE office SET priority = 31 WHERE state = 'TX' AND title = 'Judge - County Court at Law';
UPDATE office SET priority = 32 WHERE state = 'TX' AND title = 'Judge - 1st Multicounty Court at Law';
UPDATE office SET priority = 35 WHERE state = 'TX' AND title = 'Judge - County Civil Court at Law';
UPDATE office SET priority = 36 WHERE state = 'TX' AND title = 'Judge - County Criminal Court of Appeals';
UPDATE office SET priority = 37 WHERE state = 'TX' AND title = 'Judge - County Criminal Court at Law';
UPDATE office SET priority = 38 WHERE state = 'TX' AND title = 'Judge - Probate Court';

UPDATE office SET priority = 40 WHERE state = 'TX' AND title = 'County Attorney';
UPDATE office SET priority = 41 WHERE state = 'TX' AND title = 'District Clerk';
UPDATE office SET priority = 42 WHERE state = 'TX' AND title IN ('County Clerk', 'County & District Clerk');
UPDATE office SET priority = 44 WHERE state = 'TX' AND title = 'Sheriff';
UPDATE office SET priority = 45 WHERE state = 'TX' AND title = 'County Tax Assessor-Collector';
UPDATE office SET priority = 46 WHERE state = 'TX' AND title = 'County Treasurer';
UPDATE office SET priority = 47 WHERE state = 'TX' AND title = 'County Surveyor';
UPDATE office SET priority = 48 WHERE state = 'TX' AND title = 'County School Trustee';

UPDATE office SET priority = 50 WHERE state = 'TX' AND title = 'County Commissioner';
UPDATE office SET priority = 51 WHERE state = 'TX' AND title = 'Justice of the Peace';
UPDATE office SET priority = 52 WHERE state = 'TX' AND title = 'County Constable';

UPDATE office SET priority = 55 WHERE state = 'TX' AND title IN ('County Chair (D)', 'County Chair (R)');
UPDATE office SET priority = 56 WHERE state = 'TX' AND title IN ('Precinct Chair (D)', 'Precinct Chair (R)');

-- City
UPDATE office SET priority = 60 WHERE state = 'TX' AND title = 'Mayor';
UPDATE office SET priority = 61 WHERE state = 'TX' AND title = 'City Council';
