//! Texas SOS election results processor.
//!
//! Reads XML files from `scrapers/data/tx/sos` (manually downloaded from TX SoS/SFTP),
//! parses race and candidate data, and loads into `ingest_staging.stg_tx_results_sos`.

use std::fs;
use std::path::Path;
use std::str::FromStr;

use roxmltree::Document;
use sqlx::PgPool;

use crate::extractors::politician;
use crate::generators::politician::PoliticianRefKeyGenerator;

/// Default directory for TX SOS XML files (relative to scrapers crate root).
const TX_SOS_DATA_DIR: &str = "data/tx/sos";

/// One row written to ingest_staging.stg_tx_results_sos.
#[derive(Debug, Clone)]
pub struct StgTxResultRow {
    pub office_name: Option<String>,
    pub office_key: Option<String>,
    pub candidate_name: Option<String>,
    pub candidate_key: Option<String>,
    pub precincts_reporting: Option<i64>,
    pub precincts_total: Option<i64>,
    pub votes_for_candidate: Option<i64>,
    pub total_votes: Option<i64>,
    pub total_voters: Option<i64>,
    /// Party from ElectionResult.ElectionName (e.g. "democratic", "republican").
    pub party: Option<String>,
    /// Race type from ElectionResult.ElectionName (e.g. "primary" if name contains "primary").
    pub race_type: Option<String>,
    /// Year parsed from ElectionResult.ElectionDate.
    pub election_year: Option<i32>,
    /// Ref key from PoliticianRefKeyGenerator("tx-primaries", election_year, office_name, candidate_name).
    pub ref_key: String,
    /// Source file name for traceability (e.g. ResultsData_53814.xml).
    pub source_file: Option<String>,
}

/// Create ingest_staging schema and stg_tx_results_sos table.
pub async fn ensure_staging_table(pool: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ingest_staging.stg_tx_results_sos (
            id BIGSERIAL PRIMARY KEY,
            office_name TEXT,
            office_key TEXT,
            candidate_name TEXT,
            candidate_key TEXT,
            precincts_reporting BIGINT,
            precincts_total BIGINT,
            votes_for_candidate BIGINT,
            total_votes BIGINT,
            total_voters BIGINT,
            party TEXT,
            race_type TEXT,
            election_year INTEGER,
            ref_key TEXT NOT NULL,
            source_file TEXT,
            ingested_at TIMESTAMPTZ NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Return the path to the TX SOS data directory (under the scrapers crate).
fn tx_sos_data_path() -> std::path::PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    Path::new(&manifest_dir).join(TX_SOS_DATA_DIR)
}

/// List all .xml files in scrapers/data/tx/sos.
pub fn list_tx_sos_xml_files() -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    let dir = tx_sos_data_path();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files: Vec<_> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |e| e == "xml"))
        .collect();
    files.sort();
    Ok(files)
}

/// Parse a u64 from an XML attribute, returning None if missing or invalid.
fn attr_parse_u64(node: &roxmltree::Node, name: &str) -> Option<u64> {
    node.attribute(name).and_then(|s| u64::from_str(s.trim()).ok())
}

/// Normalize candidate name from XML: remove "(I)" (incumbent), strip accents, trim.
fn normalize_candidate_name(raw: &str) -> String {
    let binding = raw.trim().replace("(I)", "");
    let without_incumbent = binding.trim();
    politician::strip_accents(without_incumbent).trim().to_string()
}

/// Parse party from ElectionResult.ElectionName (e.g. "2026 DEMOCRATIC PRIMARY ELECTION" -> "democratic").
fn parse_party_from_election_name(name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    if lower.contains("republican") {
        Some("republican".to_string())
    } else if lower.contains("democratic") {
        Some("democratic".to_string())
    } else {
        None
    }
}

/// Parse race_type from ElectionResult.ElectionName: "primary" if name contains "primary".
fn parse_race_type_from_election_name(name: &str) -> Option<String> {
    if name.to_lowercase().contains("primary") {
        Some("primary".to_string())
    } else {
        None
    }
}

/// Parse year from ElectionResult.ElectionDate (e.g. "2026-03-03" -> 2026).
fn parse_year_from_election_date(date: Option<&str>) -> Option<i32> {
    let s = date?.trim();
    let year_str = s.get(0..4)?;
    year_str.parse().ok()
}

/// Parse a single TX SOS XML file and collect all candidate rows.
/// XML structure: TX-SOS > ElectionResult > Race > Candidate > County.
/// ElectionVoterTurnout nodes are ignored. Party, race_type, and election_year are read from ElectionResult.
fn parse_tx_sos_xml(content: &str, source_file: &str) -> Result<Vec<StgTxResultRow>, Box<dyn std::error::Error + Send + Sync>> {
    let doc = Document::parse(content)?;
    let root = doc.root_element();

    let mut rows = Vec::new();

    for election_result in root.children().filter(|n| n.has_tag_name("ElectionResult")) {
        let election_name = election_result.attribute("ElectionName").unwrap_or("");
        let election_date = election_result.attribute("ElectionDate");
        let party = parse_party_from_election_name(election_name);
        let race_type = parse_race_type_from_election_name(election_name);
        let election_year = parse_year_from_election_date(election_date);

        for race in election_result.children().filter(|n| n.has_tag_name("Race")) {
            let office_name = race.attribute("name").map(|s| s.trim().to_string());
            let office_key = race.attribute("key").map(String::from);

            let precincts_reporting = race
                .attribute("precinctsReported")
                .and_then(|s| u64::from_str(s.trim()).ok())
                .map(|u| u as i64);
            let precincts_total = race
                .attribute("precinctsParticipating")
                .and_then(|s| u64::from_str(s.trim()).ok())
                .map(|u| u as i64);

            let candidates: Vec<_> = race
                .children()
                .filter(|n| n.has_tag_name("Candidate"))
                .collect();

            if candidates.is_empty() {
                continue;
            }

            let candidate_votes: Vec<i64> = candidates
                .iter()
                .map(|c| {
                    let early = attr_parse_u64(c, "earlyBallotsCast").unwrap_or(0);
                    let day = attr_parse_u64(c, "ballotsCast").unwrap_or(0);
                    (early + day) as i64
                })
                .collect();
            let total_votes_race: i64 = candidate_votes.iter().sum();

            for (c, votes_for_candidate) in candidates.iter().zip(candidate_votes.iter().copied()) {
                let candidate_name = c.attribute("name").map(normalize_candidate_name);
                let candidate_key = c.attribute("key").map(String::from);
                let total_voters = attr_parse_u64(c, "totalVoters").map(|u| u as i64);

                let ref_key = PoliticianRefKeyGenerator::new(
                    "tx-primaries",
                    election_year.unwrap_or(0),
                    office_name.as_deref().unwrap_or(""),
                    candidate_name.as_deref(),
                )
                .generate();

                rows.push(StgTxResultRow {
                    office_name: office_name.clone(),
                    office_key: office_key.clone(),
                    candidate_name,
                    candidate_key,
                    precincts_reporting,
                    precincts_total,
                    votes_for_candidate: Some(votes_for_candidate),
                    total_votes: Some(total_votes_race),
                    total_voters,
                    party: party.clone(),
                    race_type: race_type.clone(),
                    election_year,
                    ref_key,
                    source_file: Some(source_file.to_string()),
                });
            }
        }
    }

    Ok(rows)
}

/// Insert a batch of staging rows into ingest_staging.stg_tx_results_sos.
pub async fn insert_staging_rows(
    pool: &PgPool,
    rows: &[StgTxResultRow],
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    if rows.is_empty() {
        return Ok(0);
    }

    let mut count = 0u64;
    for row in rows {
        sqlx::query(
            r#"
            INSERT INTO ingest_staging.stg_tx_results_sos (
                office_name, office_key, candidate_name, candidate_key,
                precincts_reporting, precincts_total, votes_for_candidate, total_votes, total_voters,
                party, race_type, election_year, ref_key, source_file
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(&row.office_name)
        .bind(&row.office_key)
        .bind(&row.candidate_name)
        .bind(&row.candidate_key)
        .bind(row.precincts_reporting)
        .bind(row.precincts_total)
        .bind(row.votes_for_candidate)
        .bind(row.total_votes)
        .bind(row.total_voters)
        .bind(&row.party)
        .bind(&row.race_type)
        .bind(row.election_year)
        .bind(&row.ref_key)
        .bind(&row.source_file)
        .execute(pool)
        .await?;
        count += 1;
    }
    Ok(count)
}

/// Process all XML files in scrapers/data/tx/sos and load into ingest_staging.stg_tx_results_sos.
/// If `clear_before_load` is true, truncates stg_tx_results_sos before inserting.
pub async fn process_tx_sos_results(
    pool: &PgPool,
    clear_before_load: bool,
) -> Result<(usize, u64), Box<dyn std::error::Error + Send + Sync>> {
    ensure_staging_table(pool).await?;

    if clear_before_load {
        sqlx::query("TRUNCATE ingest_staging.stg_tx_results_sos")
            .execute(pool)
            .await?;
    }

    let files = list_tx_sos_xml_files()?;
    if files.is_empty() {
        println!("No XML files found in {}", tx_sos_data_path().display());
        return Ok((0, 0));
    }

    let mut files_processed = 0usize;
    let mut total_rows: u64 = 0;

    for path in &files {
        let content = fs::read_to_string(path)?;
        let source_file = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let rows = parse_tx_sos_xml(&content, &source_file)?;
        let n = insert_staging_rows(pool, &rows).await?;
        files_processed += 1;
        total_rows += n;
        println!("  {}: {} rows", source_file, n);
    }

    Ok((files_processed, total_rows))
}
