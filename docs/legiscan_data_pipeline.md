
### Source Data
Legiscan bill datasets are available as CSV or JSON files [here](https://legiscan.com/datasets)

Each state has files available for the current legislative session.  The zip file for csv data breaks expands into the following files: 
`bills.csv, history.csv, sponsors.csv, rollcalls.csv, votes.csv, people.csv, documents.csv, README.md`

These files can be used to ELT into our Postgres instances.

### Scope  
1. Import (upsert) Legiscan bill data into `bill table`
2. Create new office records for the "people" if they do not exist (this will be the case for all states besides CO and MN as of now.)  You can use the `role` and `district` columns on the source "people" table to parse out the new values for our `office` table
3. Import (upsert) Legiscan "people" data into `politician` table and join the newly created `office`  via the `office_id` column on the politician
4. Use Legiscan "people.csv" data and "sponsors.csv" data to populate our `bill_sponsors` table with new join records
5. Add `bill_issue_tags` records to join our new bills with issue tags.  Note: this will require some thought as this data is not readily available from the Legiscan data sets.
```
bill, politician, bill_sponsors, bill_issue_tags* 
``` 

*The issue tags are not easily available from Legiscan data and will need some key word parsing or other to populate.  

## Specifics
I have been using a ELT (Extract, Load, Transform) approach to importing this data.  The process has been: 
- Create a new schema IF NOT EXISTS for the given state using the `p6t_state_co` naming convention
- Download the csv dataset from Legiscan for the given state [here](https://legiscan.com/datasets)
- Unzip the file and import `bills.csv, history.csv, sponsors.csv, rollcalls.csv, votes.csv, people.csv, documents.csv` as their own new tables in the new schema. I have been using the naming convention: `legiscan_state_entity_session` which would look like this for the example of California: 
```
legiscan_ca_bills_2021_2022
legiscan_ca_people_2021_2022
legiscan_ca_history_2021_2022
legiscan_ca_rollcalls_2021_2022
legiscan_ca_sponsors_2021_2022
legiscan_ca_votes_2021_2022
``` 

I like to create a .dump file for these once they are inserted that can be used for a more streamlined import to staging and production databases.

Once we have the source data loaded into our db (or your local db to start), we can use the magic of SQL to create, update, and transform the records to our needs. Here is an example query inserting the bills for California data: 
```postgresql
-- Upsert CA bills.  
INSERT INTO bill (slug, title, bill_number, legislation_status, description, full_text_url, legiscan_bill_id, legiscan_session_id, legiscan_committee_id, legiscan_committee, legiscan_last_action, legiscan_last_action_date, state)
SELECT 
	slugify(CONCAT('ca', ' ', bill_number)),
	title,
	bill_number,
	-- Note we should follow on and update status that are set here as 'passed_senate' based on the last action (if the action was passed house, it should be updated to 'passed_house')
	COALESCE(((json_build_object(1, 'introduced', 2, 'passed_senate', 4, 'became_law')::jsonb) ->> (status::text))::legislation_status, 'introduced'),
	description,
	state_link,
	bill_id,
	session_id,
	committee_id,
	committee,
	last_action,
	last_action_date::date,
	'CA' 
FROM
	p6t_state_ca.legiscan_ca_bills_2021_2022
ON CONFLICT (legiscan_bill_id) DO UPDATE
SET 
  slug = EXCLUDED.slug,
  title = EXCLUDED.title,
  bill_number = EXCLUDED.bill_number,
  legislation_status = EXCLUDED.legislation_status,
  description = EXCLUDED.description,
  full_text_url = EXCLUDED.full_text_url,
  legiscan_last_action = EXCLUDED.legiscan_last_action,
  legiscan_last_action_date = EXCLUDED.legiscan_last_action_date
RETURNING *;
```

And an example query inserting bill sponsor records:
```postgresql
-- Upsert bill sponsors
INSERT INTO bill_sponsors (politician_id, bill_id) SELECT
	p.id, b.id
FROM
	p6t_state_mn.legiscan_mn_sponsors_2021 ls
	JOIN bill b ON b.legiscan_bill_id = ls.bill_id
	JOIN politician p ON p.legiscan_people_id = ls.people_id
ON CONFLICT DO NOTHING;
```

I've left out the more difficult SQL script to create the new offices, as it is a more complicated and will require clever use of CTE (WITH statement) to insert the new offices, insert the new politicians, and join the new office to the politician via the newly created `office_id`

### Other Data Requirements (deferred for now)
- We would like to create a new `committee` table to capture legislative committee records and jon them to bills and politicians (schema tbd but should closely track Legiscans)
- We would also like to create a new `session` table to track legislative sessions.  (schema tbd but should closely track Legiscans)
- `history` - we currently have a jsonb column on the `bill` table that tracks legiscan bill action history but this would likely be better served by a new `bill_history` table so that we can enforce referential integrity and improve query speeds.  This schema is TBD

### Data Synchronization
- Legiscan offers its API that will allow us to keep our bill records up to date.  We ultimately need  a system to query Legiscan and update all our bill records on a chron job (or other queuing system).