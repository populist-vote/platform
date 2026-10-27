-- no-transaction

CREATE INDEX CONCURRENTLY IF NOT EXISTS "tx_map_muni_v1-1_set_srid_geom_idx"
ON p6t_state_tx."tx_map_muni_v1-1"
USING GIST (ST_SetSRID(geom, 3081));
