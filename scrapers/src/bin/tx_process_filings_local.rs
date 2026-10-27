//! Texas **local** candidate filings: reads `p6t_state_tx.tx_2026_local_filings_nov3`
//! into `ingest_staging.stg_tx_local_*`. Implement `process_tx_local_office` in
//! `tx_candidate_filings_local` before expecting successful inserts.
//!
//! Election id for races: `6138cc76-f273-43cf-a017-a98d1119b0c3` (`TX_LOCAL_ELECTION_ID`).

use scrapers::processors::tx::tx_candidate_filings_local::process_tx_local_filings;

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    println!("=== TX Local Candidate Filings Processor ===\n");

    match process_tx_local_filings(&pool.connection).await {
        Ok(_) => {
            println!("\n✓ Local processing finished (check error counts if office mapping is not implemented).");
        }
        Err(e) => {
            eprintln!("\n✗ Error: {}", e);
            std::process::exit(1);
        }
    }
}
