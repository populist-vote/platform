-- no-transaction

CREATE INDEX CONCURRENTLY IF NOT EXISTS tx_congressional_planc2333_set_srid_geom_idx
ON p6t_state_tx.tx_congressional_planc2333
USING GIST (ST_SetSRID(geom, 3081));
