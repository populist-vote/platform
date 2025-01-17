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
			slug = 'special-primary-for-sd-60-2025')
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
	ge.slug = 'minnesota-special-election-jan-28-2025'
),
insert_general_race_candidates AS (
    -- New CTE to insert general race candidates for winners
    INSERT INTO race_candidates (
        race_id, 
        candidate_id, 
        votes, 
        created_at, 
        updated_at
    )
    SELECT DISTINCT
        gr.general_race_id,
        unnest(w.winner_ids),  -- Explode the winner_ids array
        0 AS votes,
        NOW() AS created_at,
        NOW() AS updated_at
    FROM 
        winners w
        CROSS JOIN general_races gr
        JOIN race pr ON pr.id = w.race_id
        JOIN office po ON pr.office_id = po.id
        JOIN office go ON gr.office_id = go.id
    WHERE 
        -- Match offices in primary and general races
        po.id = go.id
    ON CONFLICT (race_id, candidate_id) DO NOTHING
    RETURNING *
)
SELECT * FROM insert_general_race_candidates;

```
