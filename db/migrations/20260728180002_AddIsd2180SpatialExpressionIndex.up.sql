-- no-transaction

CREATE INDEX CONCURRENTLY IF NOT EXISTS isd2180_set_srid_geom_idx
ON p6t_state_mn.isd2180
USING GIST (ST_SetSRID(geom, 26915));
