//! Texas candidate filings processor: reads from p6t_state_tx."tx-primaries-2026-02-09"
//! and populates ingest_staging staging tables (stg_tx_offices, stg_tx_politicians, stg_tx_races, stg_tx_race_candidates).
//! Merge to production via mn_merge_staging_to_production (or a TX-specific merge) after this.

use scrapers::processors::tx::candidate_filings::process_tx_candidate_filings;

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    println!("=== TX Candidate Filings Processor ===\n");

    match process_tx_candidate_filings(&pool.connection, "primary").await {
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
