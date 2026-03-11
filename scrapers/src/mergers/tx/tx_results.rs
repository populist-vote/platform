//! Merge ingest_staging TX staging tables (SOS, Clarity, Hart, Other) into production (race_candidates.votes, race totals).
//! Uses a single CTE-based SQL per merge (like MN SoS results) instead of a per-row loop.
//! Unmatched staging rows are recorded in ingest_staging.stg_tx_results_*_unmatched; supports dry-run.

use sqlx::PgPool;

/// Row read from ingest_staging.stg_tx_results_sos for merge.
#[derive(Debug, sqlx::FromRow)]
pub struct StgTxResultRow {
    pub id: i64,
    pub office_name: Option<String>,
    pub office_key: Option<String>,
    pub candidate_name: Option<String>,
    pub candidate_key: Option<String>,
    pub precincts_reporting: Option<i64>,
    pub precincts_total: Option<i64>,
    pub votes_for_candidate: Option<i64>,
    pub total_votes: Option<i64>,
    pub total_voters: Option<i64>,
    pub party: Option<String>,
    pub race_type: Option<String>,
    pub election_year: Option<i32>,
    pub ref_key: String,
    pub source_file: Option<String>,
}

/// Row returned by the CTE merge query (single row with stats).
#[derive(Debug, sqlx::FromRow)]
struct MergeStatsRow {
    staging_rows: i64,
    matched: i64,
    unmatched: i64,
    race_candidates_updated: i64,
    races_updated: i64,
}

/// Counts after a merge run.
#[derive(Debug, Default)]
pub struct MergeStats {
    pub staging_rows: usize,
    pub matched: usize,
    pub unmatched: usize,
    pub race_candidates_updated: usize,
    pub races_updated: usize,
}

/// Ensure ingest_staging schema and stg_tx_results_sos_unmatched table exist.
/// Drops the unmatched table each run so each run starts with an empty table.
async fn ensure_unmatched_table(pool: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_results_sos_unmatched")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_results_sos_unmatched (
            id BIGSERIAL PRIMARY KEY,
            ref_key TEXT NOT NULL,
            office_name TEXT,
            candidate_name TEXT,
            election_year INTEGER,
            party TEXT,
            source_file TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Merge a TX staging table into production using one CTE-based SQL (like MN SoS results).
/// - source: staging rows (optional test_merge filter).
/// - insert_unmatched: INSERT into unmatched table for rows with no race_candidates.ref_key match.
/// - matched: source INNER JOIN race_candidates.
/// - When !dry_run: UPDATE race_candidates and race FROM matched.
/// Returns stats from the final SELECT.
async fn merge_staging_to_production_cte(
    pool: &PgPool,
    staging_table: &str,
    unmatched_table: &str,
    dry_run: bool,
    test_merge: bool,
) -> Result<MergeStats, Box<dyn std::error::Error + Send + Sync>> {
    let source_filter = if test_merge {
        " WHERE office_name ILIKE 'U. S. Senator'"
    } else {
        ""
    };
    let source_cte = format!(
        "SELECT id, office_name, office_key, candidate_name, candidate_key, precincts_reporting, precincts_total, votes_for_candidate, total_votes, total_voters, party, race_type, election_year, ref_key, source_file FROM ingest_staging.{}",
        staging_table
    );
    let source_cte = format!("{}{}", source_cte, source_filter);

    let (update_ctes, stats_select) = if dry_run {
        (
            String::new(),
            r#"
SELECT
  (SELECT count(*)::bigint FROM source) AS staging_rows,
  (SELECT count(*)::bigint FROM matched) AS matched,
  (SELECT count(*)::bigint FROM insert_unmatched) AS unmatched,
  0::bigint AS race_candidates_updated,
  0::bigint AS races_updated
"#
            .to_string(),
        )
    } else {
        (
            format!(
                r#",
update_race_candidates AS (
  UPDATE race_candidates rc
  SET votes = m.votes_for_candidate::integer
  FROM matched m
  WHERE rc.ref_key = m.ref_key
  RETURNING rc.ref_key
),
update_race AS (
  UPDATE race r
  SET total_votes = COALESCE(m.total_votes::integer, r.total_votes),
      num_precincts_reporting = COALESCE(m.precincts_reporting::integer, r.num_precincts_reporting),
      total_precincts = COALESCE(m.precincts_total::integer, r.total_precincts)
  FROM matched m
  WHERE r.id = m.race_id
  RETURNING r.id
)"#
            ),
            r#"
SELECT
  (SELECT count(*)::bigint FROM source) AS staging_rows,
  (SELECT count(*)::bigint FROM matched) AS matched,
  (SELECT count(*)::bigint FROM insert_unmatched) AS unmatched,
  (SELECT count(*)::bigint FROM update_race_candidates) AS race_candidates_updated,
  (SELECT count(*)::bigint FROM update_race) AS races_updated
"#
            .to_string(),
        )
    };

    let query = format!(
        r#"
WITH source AS (
  {}
),
insert_unmatched AS (
  INSERT INTO ingest_staging.{} (ref_key, office_name, candidate_name, election_year, party, source_file)
  SELECT ref_key, office_name, candidate_name, election_year, party, source_file
  FROM source
  WHERE NOT EXISTS (SELECT 1 FROM race_candidates rc WHERE rc.ref_key = source.ref_key)
  RETURNING id
),
matched AS (
  SELECT source.*, rc.race_id
  FROM source
  INNER JOIN race_candidates rc ON rc.ref_key = source.ref_key
){}
{}
"#,
        source_cte,
        unmatched_table,
        update_ctes,
        stats_select
    );

    let row: MergeStatsRow = sqlx::query_as(&query).fetch_one(pool).await?;
    Ok(MergeStats {
        staging_rows: row.staging_rows as usize,
        matched: row.matched as usize,
        unmatched: row.unmatched as usize,
        race_candidates_updated: row.race_candidates_updated as usize,
        races_updated: row.races_updated as usize,
    })
}

/// Ensure ingest_staging schema and stg_tx_results_clarity_unmatched table exist.
async fn ensure_unmatched_table_clarity(pool: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_results_clarity_unmatched")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_results_clarity_unmatched (
            id BIGSERIAL PRIMARY KEY,
            ref_key TEXT NOT NULL,
            office_name TEXT,
            candidate_name TEXT,
            election_year INTEGER,
            party TEXT,
            source_file TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Ensure ingest_staging schema and stg_tx_results_hart_unmatched table exist.
async fn ensure_unmatched_table_hart(pool: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_results_hart_unmatched")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_results_hart_unmatched (
            id BIGSERIAL PRIMARY KEY,
            ref_key TEXT NOT NULL,
            office_name TEXT,
            candidate_name TEXT,
            election_year INTEGER,
            party TEXT,
            source_file TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Ensure ingest_staging schema and stg_tx_results_other_unmatched table exist.
async fn ensure_unmatched_table_other(pool: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_results_other_unmatched")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_results_other_unmatched (
            id BIGSERIAL PRIMARY KEY,
            ref_key TEXT NOT NULL,
            office_name TEXT,
            candidate_name TEXT,
            election_year INTEGER,
            party TEXT,
            source_file TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Merge ingest_staging.stg_tx_results_sos into production.
/// - Match by ref_key to race_candidates; update race_candidates.votes and race (total_votes, num_precincts_reporting, total_precincts).
/// - Rows with no matching ref_key are recorded in ingest_staging.stg_tx_results_sos_unmatched.
/// - When dry_run is true, no updates are written to production (race_candidates, race), but unmatched rows are still inserted into stg_tx_results_sos_unmatched.
/// - When test_merge is true, only rows with office_name = "U. S. Senator" are processed.
/// Uses a single CTE-based SQL (like MN SoS results) instead of a per-row loop.
pub async fn merge_stg_tx_results_sos_to_production(
    pool: &PgPool,
    dry_run: bool,
    test_merge: bool,
) -> Result<MergeStats, Box<dyn std::error::Error + Send + Sync>> {
    ensure_unmatched_table(pool).await?;
    merge_staging_to_production_cte(
        pool,
        "stg_tx_results_sos",
        "stg_tx_results_sos_unmatched",
        dry_run,
        test_merge,
    )
    .await
}

/// Merge ingest_staging.stg_tx_results_clarity into production.
/// Match by ref_key; update race_candidates.votes; update race with total_votes, num_precincts_reporting (from staging precincts_reporting), and total_precincts (from staging precincts_total).
/// Unmatched rows are recorded in ingest_staging.stg_tx_results_clarity_unmatched.
/// Uses a single CTE-based SQL instead of a per-row loop.
pub async fn merge_stg_tx_results_clarity_to_production(
    pool: &PgPool,
    dry_run: bool,
    test_merge: bool,
) -> Result<MergeStats, Box<dyn std::error::Error + Send + Sync>> {
    ensure_unmatched_table_clarity(pool).await?;
    merge_staging_to_production_cte(
        pool,
        "stg_tx_results_clarity",
        "stg_tx_results_clarity_unmatched",
        dry_run,
        test_merge,
    )
    .await
}

/// Merge ingest_staging.stg_tx_results_hart into production.
/// Same logic as merge_stg_tx_results_clarity_to_production: match by ref_key, update race_candidates.votes and race totals;
/// unmatched rows are recorded in ingest_staging.stg_tx_results_hart_unmatched.
pub async fn merge_stg_tx_results_hart_to_production(
    pool: &PgPool,
    dry_run: bool,
    test_merge: bool,
) -> Result<MergeStats, Box<dyn std::error::Error + Send + Sync>> {
    ensure_unmatched_table_hart(pool).await?;
    merge_staging_to_production_cte(
        pool,
        "stg_tx_results_hart",
        "stg_tx_results_hart_unmatched",
        dry_run,
        test_merge,
    )
    .await
}

/// Merge ingest_staging.stg_tx_results_other into production.
/// Match by ref_key; update race_candidates.votes; update race from staging: total_votes → race.total_votes, precincts_reporting → num_precincts_reporting, precincts_total → total_precincts.
/// Unmatched rows are recorded in ingest_staging.stg_tx_results_other_unmatched.
/// Uses a single CTE-based SQL instead of a per-row loop.
pub async fn merge_stg_tx_results_other_to_production(
    pool: &PgPool,
    dry_run: bool,
    test_merge: bool,
) -> Result<MergeStats, Box<dyn std::error::Error + Send + Sync>> {
    ensure_unmatched_table_other(pool).await?;
    merge_staging_to_production_cte(
        pool,
        "stg_tx_results_other",
        "stg_tx_results_other_unmatched",
        dry_run,
        test_merge,
    )
    .await
}
