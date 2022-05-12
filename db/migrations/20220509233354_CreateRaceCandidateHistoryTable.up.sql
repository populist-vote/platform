-- Add up migration script here
CREATE TABLE race_candidates (
    race_id uuid NOT NULL,
    candidate_id uuid NOT NULL,
    is_running BOOLEAN NOT NULL DEFAULT (true),
    date_announced DATE,
    date_qualified DATE,
    date_dropped DATE,
    reason_dropped TEXT,
    qualification_method TEXT,
    qualification_info TEXT,
    created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    CONSTRAINT fk_race FOREIGN KEY(race_id) REFERENCES race(id),
    CONSTRAINT fk_candidate FOREIGN KEY(candidate_id) REFERENCES politician(id),
    CONSTRAINT if_date_dropped_then_is_running_false CHECK (date_dropped IS NULL OR is_running = false)
);

INSERT INTO race_candidates (
	race_id,
	candidate_id
)
	SELECT upcoming_race_id, id FROM politician
	WHERE upcoming_race_id IS NOT NULL
;

