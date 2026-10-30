//! Merges `ingest_staging.stg_tx_local_ballot_measures` into `ballot_measure` by slug.
//! Run after `tx_process_ballot_measures`.

use db::{
    BallotMeasure, BallotMeasureStatus, DistrictType, ElectionScope, State, UpsertBallotMeasureInput,
};
use sqlx::PgPool;
use std::str::FromStr;

#[derive(sqlx::FromRow, Debug)]
struct StgBallotMeasure {
    slug: String,
    title: String,
    ballot_measure_code: String,
    description: Option<String>,
    full_text_url: Option<String>,
    status: String,
    state: String,
    county: Option<String>,
    municipality: Option<String>,
    election_scope: Option<String>,
    district_type: Option<String>,
    district: Option<String>,
    subdistrict: Option<String>,
    election_id: uuid::Uuid,
}

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    println!("=== Merge TX local ballot measures → production ===\n");

    if let Err(error) = run_merge(&pool.connection).await {
        eprintln!("Merge failed: {}", error);
        std::process::exit(1);
    }
    println!("\n✓ Merge completed successfully.");
}

async fn run_merge(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    create_audit_table(pool).await?;

    let staged: Vec<StgBallotMeasure> = sqlx::query_as(
        r#"
        SELECT slug, title, ballot_measure_code, description, full_text_url, status, state,
               county, municipality, election_scope, district_type, district, subdistrict,
               election_id
        FROM ingest_staging.stg_tx_local_ballot_measures
        ORDER BY slug
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut existing = 0usize;
    let mut inserted = 0usize;
    for stg in &staged {
        let existed = slug_exists(pool, &stg.slug).await?;
        if existed {
            existing += 1;
            insert_audit(pool, stg).await?;
        } else {
            inserted += 1;
        }
        let input = stg_to_upsert(stg)?;
        BallotMeasure::upsert_from_source(pool, &input).await?;
    }

    println!(
        "  Ballot measures: {} existing (matched by slug), {} new",
        existing, inserted
    );
    if existing > 0 {
        println!(
            "  Updated slugs recorded in ingest_staging.audit_local_ballot_measure_merge_updated"
        );
    }
    Ok(())
}

fn stg_to_upsert(
    stg: &StgBallotMeasure,
) -> Result<UpsertBallotMeasureInput, Box<dyn std::error::Error>> {
    let state = State::from_str(stg.state.trim()).map_err(|_| {
        format!(
            "ballot measure {} has unknown state {}",
            stg.slug, stg.state
        )
    })?;
    let status = parse_status(&stg.status).ok_or_else(|| {
        format!(
            "ballot measure {} has unknown status {}",
            stg.slug, stg.status
        )
    })?;
    Ok(UpsertBallotMeasureInput {
        id: None,
        slug: Some(stg.slug.clone()),
        election_id: Some(stg.election_id),
        title: Some(stg.title.clone()),
        status: Some(status),
        state: Some(state),
        county: stg.county.clone(),
        municipality: stg.municipality.clone(),
        school_district: None,
        ballot_measure_code: Some(stg.ballot_measure_code.clone()),
        measure_type: None,
        definitions: None,
        description: stg.description.clone(),
        official_summary: None,
        populist_summary: None,
        full_text_url: stg.full_text_url.clone(),
        county_fips: None,
        municipality_fips: None,
        election_scope: stg
            .election_scope
            .as_deref()
            .and_then(|scope| ElectionScope::from_str(scope.trim()).ok()),
        district_type: stg
            .district_type
            .as_deref()
            .and_then(|kind| DistrictType::from_str(kind.trim()).ok()),
        district: stg.district.clone(),
        subdistrict: stg.subdistrict.clone(),
    })
}

fn parse_status(status: &str) -> Option<BallotMeasureStatus> {
    match status.trim() {
        "OnTheBallot" => Some(BallotMeasureStatus::OnTheBallot),
        "InConsideration" => Some(BallotMeasureStatus::InConsideration),
        "Proposed" => Some(BallotMeasureStatus::Proposed),
        "Introduced" => Some(BallotMeasureStatus::Introduced),
        "GatheringSignatures" => Some(BallotMeasureStatus::GatheringSignatures),
        "BecameLaw" => Some(BallotMeasureStatus::BecameLaw),
        "Failed" => Some(BallotMeasureStatus::Failed),
        "Unknown" => Some(BallotMeasureStatus::Unknown),
        _ => None,
    }
}

async fn slug_exists(pool: &PgPool, slug: &str) -> Result<bool, sqlx::Error> {
    let row: Option<(i32,)> = sqlx::query_as("SELECT 1 FROM ballot_measure WHERE slug = $1 LIMIT 1")
        .bind(slug)
        .fetch_optional(pool)
        .await?;
    Ok(row.is_some())
}

async fn create_audit_table(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;
    sqlx::query(
        "DROP TABLE IF EXISTS ingest_staging.audit_local_ballot_measure_merge_updated CASCADE",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.audit_local_ballot_measure_merge_updated (
            slug TEXT NOT NULL,
            title TEXT NOT NULL,
            ballot_measure_code TEXT NOT NULL,
            district TEXT,
            subdistrict TEXT,
            election_scope TEXT
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_audit(pool: &PgPool, stg: &StgBallotMeasure) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.audit_local_ballot_measure_merge_updated (
            slug, title, ballot_measure_code, district, subdistrict, election_scope
        ) VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(&stg.slug)
    .bind(&stg.title)
    .bind(&stg.ballot_measure_code)
    .bind(&stg.district)
    .bind(&stg.subdistrict)
    .bind(&stg.election_scope)
    .execute(pool)
    .await?;
    Ok(())
}
