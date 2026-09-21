CREATE SCHEMA IF NOT EXISTS ingest;
CREATE SCHEMA IF NOT EXISTS ingest_staging;

CREATE TABLE ingest.source (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    authority TEXT NOT NULL,
    jurisdiction TEXT NOT NULL,
    source_url TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE ingest.run (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id TEXT NOT NULL REFERENCES ingest.source(id),
    target_environment TEXT NOT NULL CHECK (
        target_environment IN ('local', 'development', 'test', 'staging', 'production')
    ),
    run_kind TEXT NOT NULL DEFAULT 'snapshot' CHECK (
        run_kind IN ('snapshot', 'normalize', 'merge', 'promotion')
    ),
    status TEXT NOT NULL DEFAULT 'running' CHECK (
        status IN ('running', 'succeeded', 'failed', 'needs_review', 'approved', 'applied', 'verified', 'rejected')
    ),
    parser_version TEXT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at TIMESTAMPTZ,
    record_count INTEGER,
    summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    error TEXT
);

CREATE INDEX ingest_run_source_started_idx
    ON ingest.run (source_id, started_at DESC);
CREATE INDEX ingest_run_status_idx ON ingest.run (status);

CREATE TABLE ingest.artifact (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id UUID NOT NULL REFERENCES ingest.run(id) ON DELETE CASCADE,
    source_url TEXT NOT NULL,
    content_type TEXT NOT NULL,
    content_sha256 TEXT NOT NULL CHECK (length(content_sha256) = 64),
    content BYTEA NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (run_id, content_sha256)
);

CREATE INDEX ingest_artifact_sha256_idx ON ingest.artifact (content_sha256);

CREATE TABLE ingest.source_record (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id UUID NOT NULL REFERENCES ingest.run(id) ON DELETE CASCADE,
    artifact_id UUID NOT NULL REFERENCES ingest.artifact(id) ON DELETE CASCADE,
    source_record_key TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'raw' CHECK (
        status IN ('raw', 'normalized', 'ready', 'needs_review', 'excluded', 'rejected', 'applied')
    ),
    raw_record JSONB NOT NULL,
    normalized_record JSONB,
    reason_codes TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (run_id, source_record_key)
);

CREATE INDEX ingest_source_record_key_idx
    ON ingest.source_record (source_record_key);
CREATE INDEX ingest_source_record_status_idx
    ON ingest.source_record (run_id, status);

CREATE TABLE ingest.entity_alias (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id TEXT NOT NULL REFERENCES ingest.source(id),
    source_entity_key TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    target_entity_id UUID NOT NULL,
    decision TEXT NOT NULL CHECK (decision IN ('same', 'different', 'manual')),
    decided_by TEXT,
    decided_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    notes TEXT,
    UNIQUE (source_id, source_entity_key, entity_type)
);

CREATE INDEX ingest_entity_alias_target_idx
    ON ingest.entity_alias (entity_type, target_entity_id);

CREATE TABLE ingest.batch (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    election_slug TEXT NOT NULL,
    target_environment TEXT NOT NULL CHECK (
        target_environment IN ('local', 'development', 'test', 'staging', 'production')
    ),
    status TEXT NOT NULL DEFAULT 'staged' CHECK (
        status IN ('staged', 'needs_review', 'approved', 'applying', 'applied', 'verified', 'rejected', 'reverted')
    ),
    source_run_ids UUID[] NOT NULL,
    manifest_sha256 TEXT CHECK (manifest_sha256 IS NULL OR length(manifest_sha256) = 64),
    summary JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    approved_at TIMESTAMPTZ,
    approved_by TEXT,
    applied_at TIMESTAMPTZ,
    verified_at TIMESTAMPTZ
);

CREATE INDEX ingest_batch_election_created_idx
    ON ingest.batch (election_slug, created_at DESC);
CREATE INDEX ingest_batch_status_idx ON ingest.batch (status);

CREATE TABLE ingest.batch_record (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_id UUID NOT NULL REFERENCES ingest.batch(id) ON DELETE CASCADE,
    source_record_id UUID REFERENCES ingest.source_record(id),
    source_record_key TEXT NOT NULL,
    entity_type TEXT NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('insert', 'update', 'link', 'deactivate', 'exclude')),
    review_status TEXT NOT NULL DEFAULT 'pending' CHECK (
        review_status IN ('ready', 'pending', 'approved', 'rejected', 'applied')
    ),
    proposed_record JSONB NOT NULL,
    target_entity_id UUID,
    reason_codes TEXT[] NOT NULL DEFAULT '{}',
    review_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (batch_id, entity_type, source_record_key)
);

CREATE INDEX ingest_batch_record_review_idx
    ON ingest.batch_record (batch_id, review_status);

CREATE TABLE ingest.change_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_id UUID NOT NULL REFERENCES ingest.batch(id) ON DELETE CASCADE,
    entity_type TEXT NOT NULL,
    target_table TEXT NOT NULL,
    target_entity_id UUID NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('insert', 'update', 'link', 'deactivate')),
    before_record JSONB,
    after_record JSONB NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX ingest_change_log_batch_idx ON ingest.change_log (batch_id);

CREATE TABLE ingest_staging.stg_co_candidate_filings (
    source_record_key TEXT PRIMARY KEY,
    source_id TEXT NOT NULL,
    source_url TEXT NOT NULL,
    jurisdiction TEXT NOT NULL,
    election_date DATE NOT NULL,
    candidate_name TEXT NOT NULL,
    office TEXT NOT NULL,
    district TEXT,
    party TEXT,
    website TEXT,
    is_write_in BOOLEAN NOT NULL DEFAULT false,
    is_withdrawn BOOLEAN NOT NULL DEFAULT false,
    is_current BOOLEAN NOT NULL DEFAULT true,
    raw_record JSONB NOT NULL,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX stg_co_candidate_filings_current_idx
    ON ingest_staging.stg_co_candidate_filings (source_id, election_date, is_current);

CREATE TABLE ingest_staging.stg_co_municipal_candidate_filings (
    source_record_key TEXT PRIMARY KEY,
    source_id TEXT NOT NULL,
    source_url TEXT NOT NULL,
    state TEXT NOT NULL,
    municipality TEXT NOT NULL,
    election_date DATE NOT NULL,
    office TEXT NOT NULL,
    candidate_name TEXT NOT NULL,
    date_certified DATE,
    receives_matching_funds BOOLEAN NOT NULL DEFAULT false,
    is_current BOOLEAN NOT NULL DEFAULT true,
    raw_record JSONB NOT NULL,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX stg_co_municipal_candidate_filings_current_idx
    ON ingest_staging.stg_co_municipal_candidate_filings (
        source_id, election_date, municipality, is_current
    );

CREATE TABLE ingest_staging.stg_co_ballot_measures (
    source_record_key TEXT PRIMARY KEY,
    source_id TEXT NOT NULL,
    source_url TEXT NOT NULL,
    state TEXT NOT NULL,
    municipality TEXT,
    election_date DATE NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    ballot_code TEXT,
    ballot_language TEXT,
    is_final BOOLEAN NOT NULL DEFAULT false,
    is_current BOOLEAN NOT NULL DEFAULT true,
    raw_record JSONB NOT NULL,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX stg_co_ballot_measures_current_idx
    ON ingest_staging.stg_co_ballot_measures (
        source_id, election_date, municipality, is_current
    );
