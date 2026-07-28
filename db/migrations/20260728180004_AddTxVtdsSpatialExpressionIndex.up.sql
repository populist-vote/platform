-- no-transaction

CREATE INDEX CONCURRENTLY IF NOT EXISTS tx_vtds_2026_set_srid_geom_idx
ON p6t_state_tx.tx_vtds_2026
USING GIST (ST_SetSRID(geom, 3081));
