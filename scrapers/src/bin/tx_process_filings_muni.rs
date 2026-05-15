//! Texas **municipal** candidate filings: reads `p6t_state_tx.tx_2026_municipal_filings`
//! into `ingest_staging.stg_tx_muni_*`. Implement `process_tx_muni_office` in
//! `tx_candidate_filings_muni` before expecting successful inserts.
//!
//! Election id for races: `dce8e1a4-dcdb-4f3e-9cfa-11f96ae86007` (`TX_MUNI_ELECTION_ID`).

use scrapers::processors::tx::tx_candidate_filings_muni::process_tx_municipal_filings;

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    println!("=== TX Municipal Candidate Filings Processor ===\n");

    match process_tx_municipal_filings(&pool.connection).await {
        Ok(_) => {
            println!("\n✓ Municipal processing finished (check error counts if office mapping is not implemented).");
        }
        Err(e) => {
            eprintln!("\n✗ Error: {}", e);
            std::process::exit(1);
        }
    }
}
