//! Texas local ballot measures (propositions).
//!
//! Reads `p6t_state_tx.tx_2026_nov3_local_ballot_measures` into
//! `ingest_staging.stg_tx_local_ballot_measures`. Merge with `tx_merge_ballot_measures`.

use crate::extractors::tx::tx_ballot_measures::{
    extract_ballot_measure_geography, extract_measure_name, normalize_counties,
};
use crate::generators::tx::tx_ballot_measures::{
    generate_ballot_measure_code, generate_ballot_measure_slug,
};
use crate::processors::tx::tx_candidate_filings_local::TX_LOCAL_ELECTION_ID;
use db::BallotMeasureStatus;
use sqlx::FromRow;
use sqlx::PgPool;
use std::collections::HashSet;
use std::error::Error;
use uuid::Uuid;

const TX_BALLOT_MEASURE_SOURCE_TABLE: &str = "p6t_state_tx.tx_2026_nov3_local_ballot_measures";

#[derive(Debug, FromRow)]
struct TxBallotMeasureSource {
    title: Option<String>,
    description: Option<String>,
    county: Option<String>,
    political_subdivision: Option<String>,
    is_special_election: Option<bool>,
    source_url: Option<String>,
}

/// Process local ballot measures into `ingest_staging.stg_tx_local_ballot_measures`.
pub async fn process_tx_ballot_measures(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    println!("Starting TX local ballot measure processor...");
    println!("Source table: {}", TX_BALLOT_MEASURE_SOURCE_TABLE);

    create_staging_tables(pool).await?;

    let election_id = Uuid::parse_str(TX_LOCAL_ELECTION_ID)?;
    let election_slug = election_slug(pool, election_id).await?;
    println!("Election slug: {}", election_slug);

    let query = format!(
        "SELECT title, description, county, political_subdivision, is_special_election, source_url FROM {TX_BALLOT_MEASURE_SOURCE_TABLE}"
    );
    println!("Fetching ballot measures...");
    let rows: Vec<TxBallotMeasureSource> = sqlx::query_as(&query).fetch_all(pool).await?;
    println!("Found {} ballot measures to process", rows.len());

    let mut inserted = 0usize;
    let mut dupes = 0usize;
    let mut errors = 0usize;
    let mut used_slugs: HashSet<String> = HashSet::new();
    let mut seen_rows: HashSet<(String, String, String)> = HashSet::new();

    for (index, row) in rows.iter().enumerate() {
        if index % 100 == 0 {
            println!("Processing ballot measure {}/{}...", index + 1, rows.len());
        }
        match stage_ballot_measure(
            pool,
            row,
            election_id,
            &election_slug,
            &mut used_slugs,
            &mut seen_rows,
        )
        .await
        {
            Ok(StageOutcome::Inserted) => inserted += 1,
            Ok(StageOutcome::Duplicate) => dupes += 1,
            Err(error) => {
                errors += 1;
                eprintln!(
                    "Error processing ballot measure: {} — {}",
                    row.title.as_deref().unwrap_or("Unknown"),
                    error
                );
            }
        }
    }

    println!("\n=== TX local ballot measure processing complete ===");
    println!("Inserted: {}", inserted);
    println!("Duplicates: {}", dupes);
    println!("Errors: {}", errors);
    println!("\nStaging tables:");
    println!("  - ingest_staging.stg_tx_local_ballot_measures");
    println!("  - ingest_staging.stg_tx_local_ballot_measure_process_dupes");
    Ok(())
}

enum StageOutcome {
    Inserted,
    Duplicate,
}

async fn stage_ballot_measure(
    pool: &PgPool,
    row: &TxBallotMeasureSource,
    election_id: Uuid,
    election_slug: &str,
    used_slugs: &mut HashSet<String>,
    seen_rows: &mut HashSet<(String, String, String)>,
) -> Result<StageOutcome, Box<dyn Error>> {
    let source_title = row
        .title
        .as_deref()
        .map(str::trim)
        .filter(|title| !title.is_empty())
        .ok_or("Missing title")?;
    let political_subdivision = row
        .political_subdivision
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .ok_or("Missing political subdivision")?;
    let description = row.description.as_deref().and_then(normalize_description);
    let identity = (
        political_subdivision.to_string(),
        source_title.to_string(),
        description.clone().unwrap_or_default(),
    );
    if !seen_rows.insert(identity) {
        insert_process_dupe(pool, row, "same subdivision, title, and description").await?;
        return Ok(StageOutcome::Duplicate);
    }

    let measure_name = extract_measure_name(source_title);
    let geography = extract_ballot_measure_geography(political_subdivision);
    let county = row
        .county
        .as_deref()
        .and_then(normalize_counties);
    let is_special_election = row.is_special_election.unwrap_or(false);
    let ballot_measure_code = generate_ballot_measure_code(&geography, is_special_election)?;
    let slug = generate_ballot_measure_slug(
        election_slug,
        &ballot_measure_code,
        &measure_name,
        used_slugs,
    );
    let source_url = row
        .source_url
        .as_deref()
        .map(str::trim)
        .filter(|url| !url.is_empty())
        .map(str::to_string);
    let now = chrono::Utc::now();

    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_local_ballot_measures (
            id, slug, title, ballot_measure_code, description, full_text_url, status,
            state, county, municipality, election_scope, district_type, district, subdistrict,
            election_id, source_title, political_subdivision, is_special_election,
            created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7,
            $8, $9, $10, $11, $12, $13, $14,
            $15, $16, $17, $18,
            $19, $20
        )
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(&slug)
    .bind(&measure_name)
    .bind(&ballot_measure_code)
    .bind(&description)
    .bind(&source_url)
    .bind(format!("{:?}", BallotMeasureStatus::OnTheBallot))
    .bind("TX")
    .bind(&county)
    .bind(&geography.municipality)
    .bind(format!("{:?}", geography.election_scope))
    .bind(geography.district_type.as_ref().map(|kind| format!("{:?}", kind)))
    .bind(&geography.district)
    .bind(&geography.subdistrict)
    .bind(election_id)
    .bind(source_title)
    .bind(political_subdivision)
    .bind(is_special_election)
    .bind(now)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(StageOutcome::Inserted)
}

/// Drops straight and curly double quotes, then collapses leftover whitespace.
fn normalize_description(description: &str) -> Option<String> {
    let cleaned: String = description
        .chars()
        .filter(|character| !matches!(character, '"' | '\u{201C}' | '\u{201D}'))
        .collect();
    let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        None
    } else {
        Some(collapsed)
    }
}

async fn insert_process_dupe(
    pool: &PgPool,
    row: &TxBallotMeasureSource,
    reason: &str,
) -> Result<(), Box<dyn Error>> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_local_ballot_measure_process_dupes (
            title, description, county, political_subdivision, is_special_election, source_url, reason
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#,
    )
    .bind(&row.title)
    .bind(&row.description)
    .bind(&row.county)
    .bind(&row.political_subdivision)
    .bind(row.is_special_election)
    .bind(&row.source_url)
    .bind(reason)
    .execute(pool)
    .await?;
    Ok(())
}

async fn election_slug(pool: &PgPool, election_id: Uuid) -> Result<String, Box<dyn Error>> {
    let slug: Option<(String,)> =
        sqlx::query_as("SELECT slug FROM election WHERE id = $1")
            .bind(election_id)
            .fetch_optional(pool)
            .await?;
    slug.map(|(slug,)| slug)
        .ok_or_else(|| format!("No election found for id {election_id}").into())
}

async fn create_staging_tables(pool: &PgPool) -> Result<(), Box<dyn Error>> {
    println!("Creating ingest_staging.stg_tx_local_ballot_measure* tables...");
    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;
    sqlx::query(
        "DROP TABLE IF EXISTS ingest_staging.stg_tx_local_ballot_measure_process_dupes CASCADE",
    )
    .execute(pool)
    .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_local_ballot_measures CASCADE")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_local_ballot_measures (
            id UUID PRIMARY KEY,
            slug TEXT NOT NULL UNIQUE,
            title TEXT NOT NULL,
            ballot_measure_code TEXT NOT NULL,
            description TEXT,
            full_text_url TEXT,
            status TEXT NOT NULL,
            state TEXT NOT NULL,
            county TEXT,
            municipality TEXT,
            election_scope TEXT,
            district_type TEXT,
            district TEXT,
            subdistrict TEXT,
            election_id UUID NOT NULL,
            source_title TEXT NOT NULL,
            political_subdivision TEXT NOT NULL,
            is_special_election BOOLEAN NOT NULL,
            created_at TIMESTAMPTZ NOT NULL,
            updated_at TIMESTAMPTZ NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_local_ballot_measure_process_dupes (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title TEXT,
            description TEXT,
            county TEXT,
            political_subdivision TEXT,
            is_special_election BOOLEAN,
            source_url TEXT,
            reason TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::normalize_description;

    #[test]
    fn description_drops_quotation_marks() {
        assert_eq!(
            normalize_description("\"Confirming Creation of the district.\"").as_deref(),
            Some("Confirming Creation of the district.")
        );
        assert_eq!(
            normalize_description("THIS IS A TAX INCREASE \"The issuance of bonds.\"").as_deref(),
            Some("THIS IS A TAX INCREASE The issuance of bonds.")
        );
        assert_eq!(
            normalize_description("“The City’s venue project.”").as_deref(),
            Some("The City’s venue project.")
        );
        assert_eq!(normalize_description(" \" \" ").as_deref(), None);
    }
}
