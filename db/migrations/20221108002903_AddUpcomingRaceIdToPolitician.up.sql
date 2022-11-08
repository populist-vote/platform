-- Add up migration script here
ALTER TABLE politician
ADD COLUMN upcoming_race_id uuid REFERENCES race(id);

UPDATE politician p
SET upcoming_race_id = r.id
FROM (SELECT race.id as id, rc.candidate_id as candidate_id FROM race
    JOIN election ON race.election_id = election.id
    JOIN race_candidates rc ON race.id = rc.race_id
WHERE
    rc.is_running = TRUE
    AND election.election_date > NOW() - INTERVAL '1 year') as r 
WHERE p.id = r.candidate_id;
  