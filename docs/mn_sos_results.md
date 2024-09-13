## To Process Results After Primaries

```sql
BEGIN;

WITH ranked_candidates AS (
	-- Rank candidates based on votes within each race
	SELECT
		rc.race_id,
		rc.candidate_id,
		rc.votes,
		r.num_elect,
		r.total_votes,
		r.race_type,
		ROW_NUMBER() OVER (PARTITION BY rc.race_id ORDER BY rc.votes DESC NULLS LAST) AS rank,
		COUNT(*) OVER (PARTITION BY rc.race_id) AS total_candidates
	FROM
		race_candidates rc
		JOIN race r ON r.id = rc.race_id
	WHERE
		r.election_id = (
		SELECT
			id
		FROM
			election
		WHERE
			slug = 'minnesota-primaries-2024')
),
winners AS (
-- Select the winners based on num_elect and most votes
SELECT
	rc.race_id,
	r.num_elect,
	o.political_scope,
	ARRAY_AGG(rc.candidate_id ORDER BY rc.rank) AS winner_ids
FROM
	ranked_candidates rc
	JOIN race r ON rc.race_id = r.id
	JOIN office o ON r.office_id = o.id
WHERE (rc.num_elect IS NOT NULL
	AND rc.rank <= rc.num_elect * 2)
	OR(rc.num_elect IS NULL
	AND o.political_scope = 'local'
	AND rc.rank <= 2)
	OR(rc.num_elect IS NULL
	AND rc.rank = 1)
GROUP BY
	rc.race_id,
	o.political_scope,
	r.num_elect
),
aggregates AS (
-- Calculate the number of wins and losses for each politician
SELECT
	p.id AS politician_id,
	COALESCE(SUM( CASE WHEN rc.candidate_id = ANY (w.winner_ids) THEN
		1
	ELSE
		0
	END),
0) AS total_wins,
	COALESCE(SUM( CASE WHEN rc.candidate_id != ALL (w.winner_ids) THEN
		1
	ELSE
		0
	END),
0) AS total_losses
FROM
	politician p
	LEFT JOIN race_candidates rc ON rc.candidate_id = p.id
	LEFT JOIN winners w ON rc.race_id = w.race_id
GROUP BY
	p.id
),
update_wins_losses AS (
UPDATE
	politician p
SET
	race_wins = p.race_wins + a.total_wins,
	race_losses = p.race_losses + a.total_losses
FROM
	aggregates a
WHERE
	p.id = a.politician_id
),
set_winner_ids AS (
-- Set winner_ids on the race
UPDATE
	race r
SET
	winner_ids = w.winner_ids
FROM
	winners w
WHERE
	r.id = w.race_id
RETURNING
	r.id
),
general_races AS (
-- Subquery to find the corresponding general races
SELECT
	gr.id AS general_race_id,
	gr.office_id,
	gr.title,
	ge.election_date,
	gr.election_id,
	o.political_scope
FROM
	race gr
	JOIN election ge ON gr.election_id = ge.id
	JOIN office o ON gr.office_id = o.id
WHERE
	ge.slug = 'general-election-2024'
)
-- Get general race candidates to be deleted
SELECT DISTINCT ON (rc.race_id,
	rc.candidate_id)
	rc.*
FROM
	race_candidates rc
	JOIN race r ON r.id = rc.race_id
	JOIN ranked_candidates rnk ON rnk.candidate_id = rc.candidate_id
	JOIN general_races general_r ON general_r.office_id = r.office_id
WHERE
	rc.candidate_id = rnk.candidate_id
	AND rnk.rank > CASE WHEN political_scope = 'local'
		AND r.num_elect IS NULL THEN
		2
	WHEN r.num_elect IS NOT NULL THEN
		r.num_elect * 2
	ELSE
		1
	END
	AND r.election_id = (
	SELECT
		id
	FROM
		election
	WHERE
		slug = 'general-election-2024');
```