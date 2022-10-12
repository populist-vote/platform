-- Add down migration script here

-- Removes GIS support on a separate schema for the MN-specific shapefiles

DROP INDEX p6t_state_mn.school_district_boundaries_geom_idx;
DROP INDEX p6t_state_mn.isd2853_geom_idx;
DROP INDEX p6t_state_mn.isd2180_geom_idx;
DROP INDEX p6t_state_mn.bdry_votingdistricts_geom_idx;
ALTER TABLE ONLY p6t_state_mn.school_district_boundaries DROP CONSTRAINT school_district_boundaries_pkey;
ALTER TABLE ONLY p6t_state_mn.isd2853 DROP CONSTRAINT isd2853_pkey;
ALTER TABLE ONLY p6t_state_mn.isd2180 DROP CONSTRAINT isd2180_pkey;
ALTER TABLE ONLY p6t_state_mn.bdry_votingdistricts DROP CONSTRAINT bdry_votingdistricts_pkey;
ALTER TABLE p6t_state_mn.school_district_boundaries ALTER COLUMN gid DROP DEFAULT;
ALTER TABLE p6t_state_mn.isd2853 ALTER COLUMN gid DROP DEFAULT;
ALTER TABLE p6t_state_mn.isd2180 ALTER COLUMN gid DROP DEFAULT;
ALTER TABLE p6t_state_mn.bdry_votingdistricts ALTER COLUMN gid DROP DEFAULT;
DROP SEQUENCE p6t_state_mn.school_district_boundaries_gid_seq;
DROP TABLE p6t_state_mn.school_district_boundaries;
DROP TABLE p6t_state_mn.precinct_school_subdistrict_crosswalk;
DROP SEQUENCE p6t_state_mn.isd2853_gid_seq;
DROP TABLE p6t_state_mn.isd2853;
DROP SEQUENCE p6t_state_mn.isd2180_gid_seq;
DROP TABLE p6t_state_mn.isd2180;
DROP SEQUENCE p6t_state_mn.bdry_votingdistricts_gid_seq;
DROP TABLE p6t_state_mn.bdry_votingdistricts;
-- DROP EXTENSION postgis;

-- SCHEMA: p6t_state_mn

DROP SCHEMA p6t_state_mn ;