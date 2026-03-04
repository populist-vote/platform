//! Texas county election results from Hart/CIRA PDFs.
//!
//! Processes every PDF in `data/tx/counties/hart/input`, converts each to PDF-style CSV
//! via tx_hart_results_pdf_processor, writes CSVs to `data/tx/counties/hart/output`, then
//! processes each CSV into ingest_staging.stg_tx_results_hart via tx_hart_results_processor.

use std::fs;
use std::path::{Path, PathBuf};

use sqlx::PgPool;

use crate::processors::tx::counties::{tx_hart_results_pdf_processor, tx_hart_results_processor};

/// Recreate staging table once per Hart scrape run (drop then create, before CSVs are processed).
pub async fn ensure_hart_staging_table(
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;

    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_results_hart")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_results_hart (
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
            county TEXT,
            ingested_at TIMESTAMPTZ NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Input directory for Hart PDFs (relative to scrapers crate root).
pub const TX_HART_INPUT_DIR: &str = "data/tx/counties/hart/input";

/// Output directory for PDF-style CSVs (relative to scrapers crate root).
pub const TX_HART_OUTPUT_DIR: &str = "data/tx/counties/hart/output";

/// Return the path to the Hart input directory (PDFs).
pub fn hart_input_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    Path::new(&manifest_dir).join(TX_HART_INPUT_DIR)
}

/// Return the path to the Hart output directory (CSVs).
pub fn hart_output_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    Path::new(&manifest_dir).join(TX_HART_OUTPUT_DIR)
}

/// Parse county from filename. Expected format: cumulative_{county}_{party}.pdf (e.g. cumulative_hays_dem.pdf).
/// Returns the county segment title-cased (e.g. "Hays"), or empty string if the format doesn't match.
fn county_from_filename(filename_stem: &str) -> String {
    let parts: Vec<&str> = filename_stem.split('_').collect();
    let raw = if parts.len() >= 2 {
        parts[1]
    } else {
        return String::new();
    };
    if raw.is_empty() {
        return String::new();
    }
    raw.chars()
        .enumerate()
        .map(|(i, c)| {
            if i == 0 {
                c.to_uppercase().to_string()
            } else {
                c.to_lowercase().to_string()
            }
        })
        .collect::<String>()
}

/// Run the Hart pipeline: process every PDF in data/tx/counties/hart/input, write PDF-style CSVs
/// to data/tx/counties/hart/output, then process each CSV into ingest_staging.stg_tx_results_hart.
/// County is parsed from each filename (cumulative_{county}_{party}.pdf) and passed to the processor.
pub async fn run(
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let input_dir = hart_input_path();
    let output_dir = hart_output_path();

    if !input_dir.is_dir() {
        eprintln!("Input directory does not exist: {}", input_dir.display());
        eprintln!("Create it and add PDF files, then run again.");
        return Ok(());
    }

    fs::create_dir_all(&output_dir)?;

    let pdf_files: Vec<PathBuf> = fs::read_dir(&input_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .map_or(false, |e| e.eq_ignore_ascii_case("pdf"))
        })
        .collect();

    if pdf_files.is_empty() {
        eprintln!("No PDF files found in {}", input_dir.display());
        return Ok(());
    }

    ensure_hart_staging_table(pool).await?;

    println!("Found {} PDF(s) in {}", pdf_files.len(), input_dir.display());
    println!("Output: {}\n", output_dir.display());

    for pdf_path in &pdf_files {
        let base = pdf_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output");
        let csv_name = format!("{}.csv", base);
        let csv_path = output_dir.join(&csv_name);
        let county = county_from_filename(base);

        println!("Processing {} -> {} (county: {})", pdf_path.file_name().unwrap_or_default().to_string_lossy(), csv_name, if county.is_empty() { "auto-detect" } else { &county });

        match tx_hart_results_pdf_processor::parse_hart_pdf_to_csv(
            pdf_path,
            Some(&csv_path),
            &county,
        ) {
            Ok(result) => {
                println!("  ✓ {} rows -> {}", result.row_count(), csv_path.display());
                let source_file = csv_path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown");
                match tx_hart_results_processor::process_hart_csv(
                    pool,
                    &csv_path,
                    source_file,
                    None,
                )
                .await
                {
                    Ok(n) => println!("  ✓ {} rows -> ingest_staging.stg_tx_results_hart", n),
                    Err(e) => eprintln!("  ✗ Staging load error: {}", e),
                }
            }
            Err(e) => {
                eprintln!("  ✗ Error: {}", e);
            }
        }
    }

    Ok(())
}
