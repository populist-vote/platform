-- Creates p6t_state_mn.mn_candidate_filings_fed_state_county_2025 with columns
-- matching the general (fed/state/county) header names from the MN SoS scraper.
-- Run: psql $DATABASE_URL -f scripts/create_mn_candidate_filings_fed_state_county_2025.sql

CREATE TABLE IF NOT EXISTS p6t_state_mn.mn_candidate_filings_fed_state_county_2025 (
    office_code text,
    candidate_name text,
    office_id text,
    office_title text,
    county_id text,
    party_abbreviation text,
    residence_street_address text,
    residence_city text,
    residence_state text,
    residence_zip text,
    campaign_address text,
    campaign_city text,
    campaign_state text,
    campaign_zip text,
    campaign_phone text,
    campaign_website text,
    campaign_email text,
    running_mate_website text,
    running_mate_email text,
    running_mate_phone text
);
