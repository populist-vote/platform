-- Minimal Texas GIS schema for fresh local databases.
--
-- Production and staging load these tables from official shapefiles. Local
-- development only needs the schema to compile SQLx queries; load real GIS data
-- separately when testing address-to-district lookups.

CREATE SCHEMA IF NOT EXISTS p6t_state_tx;

CREATE TABLE IF NOT EXISTS p6t_state_tx.tx_vtds_2026 (
    gid BIGSERIAL PRIMARY KEY,
    cntyfips TEXT,
    prec TEXT,
    pctkey TEXT,
    countyname TEXT,
    state_sd TEXT,
    state_hd TEXT,
    boe_dist TEXT,
    coa_dist TEXT,
    state_dist_court TEXT,
    ctycom_dist TEXT,
    jp_dist TEXT,
    const_dist TEXT,
    geom geometry(MultiPolygon, 3081)
);

CREATE INDEX IF NOT EXISTS tx_vtds_2026_geom_idx
    ON p6t_state_tx.tx_vtds_2026
    USING gist (geom);

CREATE INDEX IF NOT EXISTS tx_vtds_2026_set_srid_geom_idx
    ON p6t_state_tx.tx_vtds_2026
    USING gist (ST_SetSRID(geom, 3081));

CREATE TABLE IF NOT EXISTS p6t_state_tx.tx_congressional_planc2333 (
    gid BIGSERIAL PRIMARY KEY,
    cong_dist TEXT,
    geom geometry(MultiPolygon, 3081)
);

CREATE INDEX IF NOT EXISTS tx_congressional_planc2333_geom_idx
    ON p6t_state_tx.tx_congressional_planc2333
    USING gist (geom);

CREATE INDEX IF NOT EXISTS tx_congressional_planc2333_set_srid_geom_idx
    ON p6t_state_tx.tx_congressional_planc2333
    USING gist (ST_SetSRID(geom, 3081));

CREATE TABLE IF NOT EXISTS p6t_state_tx.tx_sd_v1 (
    gid TEXT PRIMARY KEY,
    name TEXT,
    name_full TEXT,
    name_short TEXT,
    sdfips TEXT,
    sdlea TEXT,
    sdtea TEXT,
    subdist TEXT,
    geom geometry(MultiPolygon)
);

CREATE INDEX IF NOT EXISTS tx_sd_v1_geom_idx
    ON p6t_state_tx.tx_sd_v1
    USING gist (geom);

CREATE INDEX IF NOT EXISTS tx_sd_v1_set_srid_geom_idx
    ON p6t_state_tx.tx_sd_v1
    USING gist (ST_SetSRID(geom, 3081));

CREATE TABLE IF NOT EXISTS p6t_state_tx.tx_map_college_v1 (
    gid INTEGER PRIMARY KEY,
    name TEXT,
    subdist TEXT,
    county TEXT,
    geom geometry(MultiPolygon)
);

CREATE INDEX IF NOT EXISTS tx_map_college_v1_geom_idx
    ON p6t_state_tx.tx_map_college_v1
    USING gist (geom);

CREATE INDEX IF NOT EXISTS tx_map_college_v1_set_srid_geom_idx
    ON p6t_state_tx.tx_map_college_v1
    USING gist (ST_SetSRID(geom, 3081));

CREATE TABLE IF NOT EXISTS p6t_state_tx."tx_map_muni_v1-1" (
    gid INTEGER PRIMARY KEY,
    "NAME" TEXT,
    subdist TEXT,
    geom geometry(MultiPolygon)
);

CREATE INDEX IF NOT EXISTS "tx_map_muni_v1-1_geom_idx"
    ON p6t_state_tx."tx_map_muni_v1-1"
    USING gist (geom);

CREATE INDEX IF NOT EXISTS "tx_map_muni_v1-1_set_srid_geom_idx"
    ON p6t_state_tx."tx_map_muni_v1-1"
    USING gist (ST_SetSRID(geom, 3081));

\ir create_mn_candidate_filings_local_2025.sql
