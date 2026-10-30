//! Texas local ballot measures: reads `p6t_state_tx.tx_2026_nov3_local_ballot_measures`
//! into `ingest_staging.stg_tx_local_ballot_measures`.
//!
//! Election id matches local filings: `6138cc76-f273-43cf-a017-a98d1119b0c3`.
//! Merge with `tx_merge_ballot_measures`.

use scrapers::processors::tx::tx_ballot_measures::process_tx_ballot_measures;

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    println!("=== TX Local Ballot Measures Processor ===\n");

    match process_tx_ballot_measures(&pool.connection).await {
        Ok(_) => {
            println!("\n✓ Ballot measure processing finished.");
            println!("\nStaging tables:");
            println!("  SELECT * FROM ingest_staging.stg_tx_local_ballot_measures;");
            println!(
                "  SELECT * FROM ingest_staging.stg_tx_local_ballot_measure_process_dupes;"
            );
        }
        Err(error) => {
            eprintln!("\n✗ Error: {}", error);
            std::process::exit(1);
        }
    }
}
