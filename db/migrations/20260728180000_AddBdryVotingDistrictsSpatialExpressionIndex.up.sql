-- no-transaction

CREATE INDEX CONCURRENTLY IF NOT EXISTS bdry_votingdistricts_set_srid_geom_idx
ON p6t_state_mn.bdry_votingdistricts
USING GIST (ST_SetSRID(geom, 26915));
