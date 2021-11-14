-- Add up migration script here

BEGIN;

CREATE EXTENSION IF NOT EXISTS fuzzystrmatch;
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE OR REPLACE FUNCTION set_updated_at()
    RETURNS TRIGGER AS
$$
BEGIN
    OLD.updated_at = NOW();
    RETURN OLD;
END;
$$ LANGUAGE 'plpgsql'
;

CREATE TYPE vote_status AS ENUM ('introduced', 'passed', 'signed', 'vetoed', 'unknown');
CREATE TYPE state AS ENUM (
    'AL',
    'AK',
    'AZ',
    'AR',
    'CA',
    'CO',
    'CT',
    'DC',
    'DE',
    'FL',
    'GA',
    'HI',
    'ID',
    'IL',
    'IN',
    'IA',
    'KS',
    'KY',
    'LA',
    'ME',
    'MD',
    'MA',
    'MI',
    'MN',
    'MS',
    'MO',
    'MT',
    'NE',
    'NV',
    'NH',
    'NJ',
    'NM',
    'NY',
    'NC',
    'ND',
    'OH',
    'OK',
    'OR',
    'PA',
    'RI',
    'SC',
    'SD',
    'TN',
    'TX',
    'UT',
    'VT',
    'VA',
    'WA',
    'WV',
    'WI',
    'WY'
);
CREATE TYPE political_party AS ENUM ('democratic', 'republican', 'libertarian', 'green', 'constitution' );

CREATE TABLE politician (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    first_name TEXT NOT NULL,
    middle_name TEXT,
    last_name TEXT NOT NULL,
    nickname TEXT,
    preferred_name TEXT,
    ballot_name TEXT,
    description TEXT,
    thumbnail_image_url TEXT,
    home_state state NOT NULL,
    website_url TEXT,
    facebook_url TEXT,
    twitter_url TEXT,
    instagram_url TEXT,
    office_party political_party,
    vote_smart_candidate_id TEXT,
    vote_smart_candidate_data JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON politician
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at()
;

CREATE TABLE organization (
    id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    slug TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    description TEXT,
    thumbnail_image_url TEXT,
    website_url TEXT,
    facebook_url TEXT,
    twitter_url TEXT,
    instagram_url TEXT,
    -- headquarters_address_id uuid FOREIGN KEY,
    email TEXT,
    headquarters_phone TEXT,
    tax_classification TEXT,  /* example 501(c)(3) */
    created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
    updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON organization
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at()
;


CREATE TABLE election (
  id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  slug TEXT NOT NULL,
  title TEXT NOT NULL,
  description TEXT,
  election_date DATE NOT NULL
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON election
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at()
;


CREATE TABLE legislation (
  name TEXT NOT NULL,
  description TEXT,
  vote_status vote_status NOT NULL,
  official_summary TEXT,
  populist_summary TEXT,
  full_text_url TEXT,
  created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
  updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON legislation
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at()
;


CREATE TABLE bill (
  id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  slug TEXT NOT NULL UNIQUE,
  legiscan_bill_id INT,
  legiscan_data JSONB NOT NULL DEFAULT '{}'::jsonb
) INHERITS (legislation);

CREATE TABLE ballot_measure (
  id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  slug TEXT NOT NULL UNIQUE,
  election_id uuid NOT NULL,
  ballot_state state NOT NULL,
  ballot_measure_code TEXT NOT NULL UNIQUE,
  measure_type TEXT NOT NULL,
  definitions TEXT NOT NULL,
  CONSTRAINT fk_election FOREIGN KEY(election_id) REFERENCES election(id)
) INHERITS (legislation);

CREATE TABLE politician_endorsements (
  id uuid NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
  politician_id uuid NOT NULL,
  organization_id uuid NOT NULL,
  created_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
  updated_at timestamptz NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
  CONSTRAINT fk_politician FOREIGN KEY(politician_id) REFERENCES politician(id),
  CONSTRAINT fk_organization FOREIGN KEY(organization_id) REFERENCES organization(id)
);

CREATE TRIGGER set_updated_at
    BEFORE UPDATE
    ON politician_endorsements
    FOR EACH ROW
EXECUTE PROCEDURE set_updated_at()
;

COMMIT;

