//! Texas candidate filings processor: reads from p6t_state_tx."tx-primaries-2026-02-09"
//! and populates ingest_staging staging tables (stg_tx_offices, stg_tx_politicians, stg_tx_races, stg_tx_race_candidates).
//! Merge to production via mn_merge_staging_to_production (or a TX-specific merge) after this.
//!
//! Usage:
//!   cargo run --bin tx_process_filings -- --primary
//!   cargo run --bin tx_process_filings -- --general

use scrapers::processors::tx::tx_candidate_filings::process_tx_candidate_filings;

fn parse_race_type(args: &[String]) -> Result<&'static str, String> {
    let want_primary = args.iter().any(|a| a == "--primary");
    let want_general = args.iter().any(|a| a == "--general");

    match (want_primary, want_general) {
        (true, false) => Ok("primary"),
        (false, true) => Ok("general"),
        (true, true) => Err("Specify only one of --primary or --general".to_string()),
        (false, false) => Err(
            "Specify --primary or --general\n\nUsage:\n  cargo run --bin tx_process_filings -- --primary\n  cargo run --bin tx_process_filings -- --general"
                .to_string(),
        ),
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let race_type = match parse_race_type(&args) {
        Ok(rt) => rt,
        Err(msg) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    };

    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    println!("=== TX Candidate Filings Processor ({}) ===\n", race_type);

    match process_tx_candidate_filings(&pool.connection, race_type).await {
        Ok(_) => {
            println!("\n✓ Processing completed successfully!");
            println!("\nStaging tables:");
            println!("  SELECT * FROM ingest_staging.stg_tx_offices;");
            println!("  SELECT * FROM ingest_staging.stg_tx_politicians;");
            println!("  SELECT * FROM ingest_staging.stg_tx_races;");
            println!("  SELECT * FROM ingest_staging.stg_tx_race_candidates;");
        }
        Err(e) => {
            eprintln!("\n✗ Error processing filings: {}", e);
            std::process::exit(1);
        }
    }
}
