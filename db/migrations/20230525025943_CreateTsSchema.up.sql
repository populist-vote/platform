-- Add up migration script here
CREATE SCHEMA IF NOT EXISTS ts;
GRANT USAGE ON SCHEMA ts TO public;
COMMENT ON SCHEMA ts IS 'text search objects';

CREATE TEXT SEARCH DICTIONARY ts.english_simple_dict (
    TEMPLATE = pg_catalog.simple
  , STOPWORDS = english
);

CREATE TEXT SEARCH CONFIGURATION ts.english_simple (COPY = simple);
ALTER  TEXT SEARCH CONFIGURATION ts.english_simple
   ALTER MAPPING FOR asciiword WITH ts.english_simple_dict;