//! Minnesota candidate filings processor.
//!
//! Usage:
//!   cargo run --bin mn_process_filings -- --local
//!   cargo run --bin mn_process_filings -- --fed-state-county

use scrapers::processors::mn::mn_candidate_filings::{
    process_mn_candidate_filings, MnFilingScope,
};

fn parse_filing_scope(args: &[String]) -> Result<MnFilingScope, String> {
    let want_local = args.iter().any(|a| a == "--local");
    let want_fed = args.iter().any(|a| a == "--fed-state-county");

    match (want_local, want_fed) {
        (true, false) => Ok(MnFilingScope::Local),
        (false, true) => Ok(MnFilingScope::FedStateCounty),
        (true, true) => Err("Specify only one of --local or --fed-state-county".to_string()),
        (false, false) => Err(
            "Specify --local or --fed-state-county\n\nUsage:\n  cargo run --bin mn_process_filings -- --local\n  cargo run --bin mn_process_filings -- --fed-state-county"
                .to_string(),
        ),
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let scope = match parse_filing_scope(&args) {
        Ok(scope) => scope,
        Err(msg) => {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
    };

    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    println!("=== MN Candidate Filings Processor ({}) ===\n", scope);

    match process_mn_candidate_filings(&pool.connection, scope, "primary").await {
        Ok(_) => {
            println!("\n✓ Processing completed successfully!");
            println!("\nYou can now examine the staging tables:");
            println!("  SELECT * FROM ingest_staging.stg_mn_offices;");
            println!("  SELECT * FROM ingest_staging.stg_mn_politicians;");
            println!("  SELECT * FROM ingest_staging.stg_mn_races;");
            println!("  SELECT * FROM ingest_staging.stg_mn_race_candidates;");
        }
        Err(e) => {
            eprintln!("\n✗ Error processing filings: {}", e);
            std::process::exit(1);
        }
    }
}
