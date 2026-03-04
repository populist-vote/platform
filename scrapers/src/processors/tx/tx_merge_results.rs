//! Merge ingest_staging.stg_tx_results_sos into production (race_candidates.votes, race totals).
//! Option B: Rust loop over staging rows; track unmatched rows in stg_tx_results_sos_unmatched;
//! supports dry-run.

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

/// Production race_candidate row identified by ref_key (for lookup).
#[derive(Debug, sqlx::FromRow)]
struct RaceCandidateRef {
    race_id: uuid::Uuid,
    candidate_id: uuid::Uuid,
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

/// Insert one row into stg_tx_results_sos_unmatched (staging row had no matching race_candidate).
async fn insert_unmatched(
    pool: &PgPool,
    row: &StgTxResultRow,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_results_sos_unmatched (ref_key, office_name, candidate_name, election_year, party, source_file)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(&row.ref_key)
    .bind(&row.office_name)
    .bind(&row.candidate_name)
    .bind(row.election_year)
    .bind(&row.party)
    .bind(&row.source_file)
    .execute(pool)
    .await?;
    Ok(())
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

/// Insert one row into stg_tx_results_clarity_unmatched (staging row had no matching race_candidate).
async fn insert_unmatched_clarity(
    pool: &PgPool,
    row: &StgTxResultRow,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_results_clarity_unmatched (ref_key, office_name, candidate_name, election_year, party, source_file)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(&row.ref_key)
    .bind(&row.office_name)
    .bind(&row.candidate_name)
    .bind(row.election_year)
    .bind(&row.party)
    .bind(&row.source_file)
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

/// Insert one row into stg_tx_results_hart_unmatched (staging row had no matching race_candidate).
async fn insert_unmatched_hart(
    pool: &PgPool,
    row: &StgTxResultRow,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_results_hart_unmatched (ref_key, office_name, candidate_name, election_year, party, source_file)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(&row.ref_key)
    .bind(&row.office_name)
    .bind(&row.candidate_name)
    .bind(row.election_year)
    .bind(&row.party)
    .bind(&row.source_file)
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

/// Insert one row into stg_tx_results_other_unmatched (staging row had no matching race_candidate).
async fn insert_unmatched_other(
    pool: &PgPool,
    row: &StgTxResultRow,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_results_other_unmatched (ref_key, office_name, candidate_name, election_year, party, source_file)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(&row.ref_key)
    .bind(&row.office_name)
    .bind(&row.candidate_name)
    .bind(row.election_year)
    .bind(&row.party)
    .bind(&row.source_file)
    .execute(pool)
    .await?;
    Ok(())
}

/// Merge ingest_staging.stg_tx_results_sos into production.
/// - Match by ref_key to race_candidates; update race_candidates.votes and race (total_votes, num_precincts_reporting, total_precincts).
/// - Rows with no matching ref_key are recorded in ingest_staging.stg_tx_results_sos_unmatched.
/// - When dry_run is true, no updates are written to production (race_candidates, race), but unmatched rows are still inserted into stg_tx_results_sos_unmatched.
/// - When test_merge is true, only rows with office_name = "U. S. Senator" are processed.
pub async fn merge_stg_tx_results_sos_to_production(
    pool: &PgPool,
    dry_run: bool,
    test_merge: bool,
) -> Result<MergeStats, Box<dyn std::error::Error + Send + Sync>> {
    ensure_unmatched_table(pool).await?;

    let rows: Vec<StgTxResultRow> = sqlx::query_as(
        "SELECT id, office_name, office_key, candidate_name, candidate_key, precincts_reporting, precincts_total, votes_for_candidate, total_votes, total_voters, party, race_type, election_year, ref_key, source_file FROM ingest_staging.stg_tx_results_sos",
    )
    .fetch_all(pool)
    .await?;

    let rows: Vec<StgTxResultRow> = if test_merge {
        rows.into_iter()
            .filter(|r| r.office_name.as_deref().is_some_and(|s| s.eq_ignore_ascii_case("U. S. Senator")))
            .collect()
    } else {
        rows
    };

    let mut stats = MergeStats {
        staging_rows: rows.len(),
        ..Default::default()
    };

    for row in &rows {
        let rc: Option<RaceCandidateRef> = sqlx::query_as(
            "SELECT race_id, candidate_id FROM race_candidates WHERE ref_key = $1",
        )
        .bind(&row.ref_key)
        .fetch_optional(pool)
        .await?;

        match rc {
            None => {
                stats.unmatched += 1;
                // Always record unmatched rows (including in dry run) for review.
                insert_unmatched(pool, row).await?;
            }
            Some(rc_ref) => {
                stats.matched += 1;
                if dry_run {
                    continue;
                }
                let votes = row.votes_for_candidate.map(|v| v as i32);
                let total_votes = row.total_votes.map(|v| v as i32);
                let precincts_reporting = row.precincts_reporting.map(|v| v as i32);
                let precincts_total = row.precincts_total.map(|v| v as i32);

                sqlx::query(
                    "UPDATE race_candidates SET votes = $1 WHERE ref_key = $2",
                )
                .bind(votes)
                .bind(&row.ref_key)
                .execute(pool)
                .await?;
                stats.race_candidates_updated += 1;

                sqlx::query(
                    r#"
                    UPDATE race
                    SET total_votes = COALESCE($1, race.total_votes),
                        num_precincts_reporting = COALESCE($2, race.num_precincts_reporting),
                        total_precincts = COALESCE($3, race.total_precincts)
                    WHERE id = $4
                    "#,
                )
                .bind(total_votes)
                .bind(precincts_reporting)
                .bind(precincts_total)
                .bind(rc_ref.race_id)
                .execute(pool)
                .await?;
                stats.races_updated += 1;
            }
        }
    }

    Ok(stats)
}

/// Merge ingest_staging.stg_tx_results_clarity into production.
/// Same logic as merge_stg_tx_results_sos_to_production: match by ref_key, update race_candidates.votes and race totals;
/// unmatched rows are recorded in ingest_staging.stg_tx_results_clarity_unmatched.
pub async fn merge_stg_tx_results_clarity_to_production(
    pool: &PgPool,
    dry_run: bool,
    test_merge: bool,
) -> Result<MergeStats, Box<dyn std::error::Error + Send + Sync>> {
    ensure_unmatched_table_clarity(pool).await?;

    let rows: Vec<StgTxResultRow> = sqlx::query_as(
        "SELECT id, office_name, office_key, candidate_name, candidate_key, precincts_reporting, precincts_total, votes_for_candidate, total_votes, total_voters, party, race_type, election_year, ref_key, source_file FROM ingest_staging.stg_tx_results_clarity",
    )
    .fetch_all(pool)
    .await?;

    let rows: Vec<StgTxResultRow> = if test_merge {
        rows.into_iter()
            .filter(|r| r.office_name.as_deref().is_some_and(|s| s.eq_ignore_ascii_case("U. S. Senator")))
            .collect()
    } else {
        rows
    };

    let mut stats = MergeStats {
        staging_rows: rows.len(),
        ..Default::default()
    };

    for row in &rows {
        let rc: Option<RaceCandidateRef> = sqlx::query_as(
            "SELECT race_id, candidate_id FROM race_candidates WHERE ref_key = $1",
        )
        .bind(&row.ref_key)
        .fetch_optional(pool)
        .await?;

        match rc {
            None => {
                stats.unmatched += 1;
                insert_unmatched_clarity(pool, row).await?;
            }
            Some(rc_ref) => {
                stats.matched += 1;
                if dry_run {
                    continue;
                }
                let votes = row.votes_for_candidate.map(|v| v as i32);
                let total_votes = row.total_votes.map(|v| v as i32);
                let precincts_reporting = row.precincts_reporting.map(|v| v as i32);
                let precincts_total = row.precincts_total.map(|v| v as i32);

                sqlx::query(
                    "UPDATE race_candidates SET votes = $1 WHERE ref_key = $2",
                )
                .bind(votes)
                .bind(&row.ref_key)
                .execute(pool)
                .await?;
                stats.race_candidates_updated += 1;

                sqlx::query(
                    r#"
                    UPDATE race
                    SET total_votes = COALESCE($1, race.total_votes),
                        num_precincts_reporting = COALESCE($2, race.num_precincts_reporting),
                        total_precincts = COALESCE($3, race.total_precincts)
                    WHERE id = $4
                    "#,
                )
                .bind(total_votes)
                .bind(precincts_reporting)
                .bind(precincts_total)
                .bind(rc_ref.race_id)
                .execute(pool)
                .await?;
                stats.races_updated += 1;
            }
        }
    }

    Ok(stats)
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

    let rows: Vec<StgTxResultRow> = sqlx::query_as(
        "SELECT id, office_name, office_key, candidate_name, candidate_key, precincts_reporting, precincts_total, votes_for_candidate, total_votes, total_voters, party, race_type, election_year, ref_key, source_file FROM ingest_staging.stg_tx_results_hart",
    )
    .fetch_all(pool)
    .await?;

    let rows: Vec<StgTxResultRow> = if test_merge {
        rows.into_iter()
            .filter(|r| r.office_name.as_deref().is_some_and(|s| s.eq_ignore_ascii_case("U. S. Senator")))
            .collect()
    } else {
        rows
    };

    let mut stats = MergeStats {
        staging_rows: rows.len(),
        ..Default::default()
    };

    for row in &rows {
        let rc: Option<RaceCandidateRef> = sqlx::query_as(
            "SELECT race_id, candidate_id FROM race_candidates WHERE ref_key = $1",
        )
        .bind(&row.ref_key)
        .fetch_optional(pool)
        .await?;

        match rc {
            None => {
                stats.unmatched += 1;
                insert_unmatched_hart(pool, row).await?;
            }
            Some(rc_ref) => {
                stats.matched += 1;
                if dry_run {
                    continue;
                }
                let votes = row.votes_for_candidate.map(|v| v as i32);
                let total_votes = row.total_votes.map(|v| v as i32);
                let precincts_reporting = row.precincts_reporting.map(|v| v as i32);
                let precincts_total = row.precincts_total.map(|v| v as i32);

                sqlx::query(
                    "UPDATE race_candidates SET votes = $1 WHERE ref_key = $2",
                )
                .bind(votes)
                .bind(&row.ref_key)
                .execute(pool)
                .await?;
                stats.race_candidates_updated += 1;

                sqlx::query(
                    r#"
                    UPDATE race
                    SET total_votes = COALESCE($1, race.total_votes),
                        num_precincts_reporting = COALESCE($2, race.num_precincts_reporting),
                        total_precincts = COALESCE($3, race.total_precincts)
                    WHERE id = $4
                    "#,
                )
                .bind(total_votes)
                .bind(precincts_reporting)
                .bind(precincts_total)
                .bind(rc_ref.race_id)
                .execute(pool)
                .await?;
                stats.races_updated += 1;
            }
        }
    }

    Ok(stats)
}

/// Merge ingest_staging.stg_tx_results_other into production.
/// Same logic as merge_stg_tx_results_hart_to_production: match by ref_key, update race_candidates.votes and race totals;
/// unmatched rows are recorded in ingest_staging.stg_tx_results_other_unmatched.
pub async fn merge_stg_tx_results_other_to_production(
    pool: &PgPool,
    dry_run: bool,
    test_merge: bool,
) -> Result<MergeStats, Box<dyn std::error::Error + Send + Sync>> {
    ensure_unmatched_table_other(pool).await?;

    let rows: Vec<StgTxResultRow> = sqlx::query_as(
        "SELECT id, office_name, office_key, candidate_name, candidate_key, precincts_reporting, precincts_total, votes_for_candidate, total_votes, total_voters, party, race_type, election_year, ref_key, source_file FROM ingest_staging.stg_tx_results_other",
    )
    .fetch_all(pool)
    .await?;

    let rows: Vec<StgTxResultRow> = if test_merge {
        rows.into_iter()
            .filter(|r| r.office_name.as_deref().is_some_and(|s| s.eq_ignore_ascii_case("U. S. Senator")))
            .collect()
    } else {
        rows
    };

    let mut stats = MergeStats {
        staging_rows: rows.len(),
        ..Default::default()
    };

    for row in &rows {
        let rc: Option<RaceCandidateRef> = sqlx::query_as(
            "SELECT race_id, candidate_id FROM race_candidates WHERE ref_key = $1",
        )
        .bind(&row.ref_key)
        .fetch_optional(pool)
        .await?;

        match rc {
            None => {
                stats.unmatched += 1;
                insert_unmatched_other(pool, row).await?;
            }
            Some(rc_ref) => {
                stats.matched += 1;
                if dry_run {
                    continue;
                }
                let votes = row.votes_for_candidate.map(|v| v as i32);
                let total_votes = row.total_votes.map(|v| v as i32);
                let precincts_reporting = row.precincts_reporting.map(|v| v as i32);
                let precincts_total = row.precincts_total.map(|v| v as i32);

                sqlx::query(
                    "UPDATE race_candidates SET votes = $1 WHERE ref_key = $2",
                )
                .bind(votes)
                .bind(&row.ref_key)
                .execute(pool)
                .await?;
                stats.race_candidates_updated += 1;

                sqlx::query(
                    r#"
                    UPDATE race
                    SET total_votes = COALESCE($1, race.total_votes),
                        num_precincts_reporting = COALESCE($2, race.num_precincts_reporting),
                        total_precincts = COALESCE($3, race.total_precincts)
                    WHERE id = $4
                    "#,
                )
                .bind(total_votes)
                .bind(precincts_reporting)
                .bind(precincts_total)
                .bind(rc_ref.race_id)
                .execute(pool)
                .await?;
                stats.races_updated += 1;
            }
        }
    }

    Ok(stats)
}
