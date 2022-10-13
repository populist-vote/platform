-- Add up migration script here

-- Adds GIS support on a separate schema for the MN-specific shapefiles

-- SCHEMA: p6t_state_mn

CREATE SCHEMA p6t_state_mn;

COMMENT ON SCHEMA p6t_state_mn
    IS 'MN state data and GIS shapefiles';

GRANT ALL ON SCHEMA p6t_state_mn TO PUBLIC;

--
-- PostgreSQL database dump
--

-- Dumped from database version 12.4 (Ubuntu 12.4-1.pgdg18.04+1)
-- Dumped by pg_dump version 12.4 (Ubuntu 12.4-1.pgdg18.04+1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
-- SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: postgis; Type: EXTENSION; Schema: -; Owner: -
--

CREATE EXTENSION IF NOT EXISTS postgis WITH SCHEMA public;


--
-- Name: EXTENSION postgis; Type: COMMENT; Schema: -; Owner: -
--

COMMENT ON EXTENSION postgis IS 'PostGIS geometry and geography spatial types and functions';


SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: bdry_votingdistricts; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE p6t_state_mn.bdry_votingdistricts (
    gid integer NOT NULL,
    vtdid character varying(12),
    pctname character varying(50),
    pctcode character varying(5),
    shortlabel character varying(254),
    mcdname character varying(50),
    mcdcode character varying(5),
    mcdfips character varying(10),
    mcdgnis character varying(12),
    ctu_type character varying(25),
    countyname character varying(25),
    countycode character varying(5),
    countyfips character varying(5),
    congdist character varying(254),
    mnsendist character varying(254),
    mnlegdist character varying(254),
    ctycomdist character varying(254),
    juddist character varying(254),
    swcdist character varying(254),
    swcdist_n character varying(254),
    ward character varying(254),
    hospdist character varying(254),
    hospdist_n character varying(254),
    parkdist character varying(254),
    parkdist_n character varying(254),
    geom public.geometry(MultiPolygon)
);


--
-- Name: bdry_votingdistricts_gid_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE p6t_state_mn.bdry_votingdistricts_gid_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: bdry_votingdistricts_gid_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE p6t_state_mn.bdry_votingdistricts_gid_seq OWNED BY p6t_state_mn.bdry_votingdistricts.gid;


--
-- Name: isd2180; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE p6t_state_mn.isd2180 (
    gid integer NOT NULL,
    id bigint,
    schsubdist character varying(25),
    sdnumber character varying(5),
    geom public.geometry(MultiPolygon)
);


--
-- Name: isd2180_gid_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE p6t_state_mn.isd2180_gid_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: isd2180_gid_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE p6t_state_mn.isd2180_gid_seq OWNED BY p6t_state_mn.isd2180.gid;


--
-- Name: isd2853; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE p6t_state_mn.isd2853 (
    gid integer NOT NULL,
    fid_school integer,
    uni_typ smallint,
    uni_maj smallint,
    uni_nam character varying(40),
    vtdid character varying(254),
    pctname character varying(254),
    pctcode character varying(254),
    shortlabel character varying(254),
    mcdname character varying(254),
    mcdcode character varying(254),
    mcdfips character varying(254),
    mcdgnis character varying(254),
    ctu_type character varying(254),
    countyname character varying(254),
    countycode character varying(254),
    countyfips character varying(254),
    schsubdist character varying(25),
    geom public.geometry(MultiPolygon)
);


--
-- Name: isd2853_gid_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE p6t_state_mn.isd2853_gid_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: isd2853_gid_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE p6t_state_mn.isd2853_gid_seq OWNED BY p6t_state_mn.isd2853.gid;


--
-- Name: precinct_school_subdistrict_crosswalk; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE p6t_state_mn.precinct_school_subdistrict_crosswalk (
    county_id character varying(5),
    precinct_code character varying(5),
    precinct_name character varying(50),
    school_district_number character varying(4),
    school_district_name character varying(100),
    school_subdistrict_code character varying(4),
    school_subdistrict_name character varying(50)
);


--
-- Name: school_district_boundaries; Type: TABLE; Schema: public; Owner: -
--

CREATE TABLE p6t_state_mn.school_district_boundaries (
    gid integer NOT NULL,
    sdorgid numeric,
    formid character varying(30),
    sdtype character varying(2),
    sdnumber character varying(4),
    prefname character varying(100),
    shortname character varying(50),
    web_url character varying(200),
    sqmiles numeric,
    acres numeric,
    shape_leng numeric,
    shape_area numeric,
    geom public.geometry(MultiPolygon)
);


--
-- Name: school_district_boundaries_gid_seq; Type: SEQUENCE; Schema: public; Owner: -
--

CREATE SEQUENCE p6t_state_mn.school_district_boundaries_gid_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


--
-- Name: school_district_boundaries_gid_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: -
--

ALTER SEQUENCE p6t_state_mn.school_district_boundaries_gid_seq OWNED BY p6t_state_mn.school_district_boundaries.gid;


--
-- Name: bdry_votingdistricts gid; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY p6t_state_mn.bdry_votingdistricts ALTER COLUMN gid SET DEFAULT nextval('p6t_state_mn.bdry_votingdistricts_gid_seq'::regclass);


--
-- Name: isd2180 gid; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY p6t_state_mn.isd2180 ALTER COLUMN gid SET DEFAULT nextval('p6t_state_mn.isd2180_gid_seq'::regclass);


--
-- Name: isd2853 gid; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY p6t_state_mn.isd2853 ALTER COLUMN gid SET DEFAULT nextval('p6t_state_mn.isd2853_gid_seq'::regclass);


--
-- Name: school_district_boundaries gid; Type: DEFAULT; Schema: public; Owner: -
--

ALTER TABLE ONLY p6t_state_mn.school_district_boundaries ALTER COLUMN gid SET DEFAULT nextval('p6t_state_mn.school_district_boundaries_gid_seq'::regclass);

--
-- Name: bdry_votingdistricts_gid_seq; Type: SEQUENCE SET; Schema: public; Owner: -
--

SELECT pg_catalog.setval('p6t_state_mn.bdry_votingdistricts_gid_seq', 4103, true);


--
-- Name: isd2180_gid_seq; Type: SEQUENCE SET; Schema: public; Owner: -
--

SELECT pg_catalog.setval('p6t_state_mn.isd2180_gid_seq', 6, true);


--
-- Name: isd2853_gid_seq; Type: SEQUENCE SET; Schema: public; Owner: -
--

SELECT pg_catalog.setval('p6t_state_mn.isd2853_gid_seq', 44, true);


--
-- Name: school_district_boundaries_gid_seq; Type: SEQUENCE SET; Schema: public; Owner: -
--

SELECT pg_catalog.setval('p6t_state_mn.school_district_boundaries_gid_seq', 329, true);


--
-- Name: bdry_votingdistricts bdry_votingdistricts_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY p6t_state_mn.bdry_votingdistricts
    ADD CONSTRAINT bdry_votingdistricts_pkey PRIMARY KEY (gid);


--
-- Name: isd2180 isd2180_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY p6t_state_mn.isd2180
    ADD CONSTRAINT isd2180_pkey PRIMARY KEY (gid);


--
-- Name: isd2853 isd2853_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY p6t_state_mn.isd2853
    ADD CONSTRAINT isd2853_pkey PRIMARY KEY (gid);


--
-- Name: school_district_boundaries school_district_boundaries_pkey; Type: CONSTRAINT; Schema: public; Owner: -
--

ALTER TABLE ONLY p6t_state_mn.school_district_boundaries
    ADD CONSTRAINT school_district_boundaries_pkey PRIMARY KEY (gid);


--
-- Name: bdry_votingdistricts_geom_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX bdry_votingdistricts_geom_idx ON p6t_state_mn.bdry_votingdistricts USING gist (geom);


--
-- Name: isd2180_geom_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX isd2180_geom_idx ON p6t_state_mn.isd2180 USING gist (geom);


--
-- Name: isd2853_geom_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX isd2853_geom_idx ON p6t_state_mn.isd2853 USING gist (geom);


--
-- Name: school_district_boundaries_geom_idx; Type: INDEX; Schema: public; Owner: -
--

CREATE INDEX school_district_boundaries_geom_idx ON p6t_state_mn.school_district_boundaries USING gist (geom);


--
-- PostgreSQL database dump complete
--
