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

CREATE TABLE IF NOT EXISTS p6t_state_tx.tx_congressional_planc2333 (
    gid BIGSERIAL PRIMARY KEY,
    cong_dist TEXT,
    geom geometry(MultiPolygon, 3081)
);

CREATE INDEX IF NOT EXISTS tx_congressional_planc2333_geom_idx
    ON p6t_state_tx.tx_congressional_planc2333
    USING gist (geom);

\ir create_mn_candidate_filings_local_2025.sql
