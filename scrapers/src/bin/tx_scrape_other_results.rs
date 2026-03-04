//! Read all CSVs in data/tx/counties/other and load into ingest_staging.stg_tx_results_other.
//! Drops and recreates the staging table once per run, then processes each CSV.

use std::fs;
use std::path::Path;

use scrapers::processors::tx::counties::tx_other_results_processor;

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;
    let db = &pool.connection;

    println!("=== TX Other County Results ===\n");
    let data_dir = tx_other_results_processor::other_data_path();
    println!("Data dir: {}\n", data_dir.display());

    if !data_dir.is_dir() {
        eprintln!("Directory does not exist: {}", data_dir.display());
        eprintln!("Create it and add CSV files, then run again.");
        std::process::exit(1);
    }

    // Recreate staging table once per run
    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(db)
        .await
        .expect("create schema");
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_results_other")
        .execute(db)
        .await
        .expect("drop table");
    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_results_other (
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
    .execute(db)
    .await
    .expect("create table");

    let csv_files: Vec<_> = fs::read_dir(&data_dir)
        .expect("read dir")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension()
                .map_or(false, |e| e.eq_ignore_ascii_case("csv"))
        })
        .collect();

    if csv_files.is_empty() {
        eprintln!("No CSV files found in {}", data_dir.display());
        std::process::exit(1);
    }

    println!("Found {} CSV(s)\n", csv_files.len());
    let mut total_rows = 0u64;
    for csv_path in &csv_files {
        let source_file = csv_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        match tx_other_results_processor::process_other_csv(
            db,
            csv_path,
            source_file,
            None,
        )
        .await
        {
            Ok(n) => {
                println!("  {}: {} rows", csv_path.display(), n);
                total_rows += n;
            }
            Err(e) => {
                eprintln!("  {}: error: {}", csv_path.display(), e);
            }
        }
    }

    println!("\n✓ Done. {} total rows -> ingest_staging.stg_tx_results_other", total_rows);
}
