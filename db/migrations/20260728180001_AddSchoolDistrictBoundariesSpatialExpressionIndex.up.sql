-- no-transaction

CREATE INDEX CONCURRENTLY IF NOT EXISTS school_district_boundaries_set_srid_geom_idx
ON p6t_state_mn.school_district_boundaries
USING GIST (ST_SetSRID(geom, 26915));
