//! Texas county election results from Clarity.
//!
//! Reads county_clarity_results_urls.csv (columns: url, county_name) from data/tx/counties
//! by default, downloads each zip (renamed with county name), unzips and renames CSVs with
//! county name, then processes each CSV into ingest_staging.stg_tx_results_clarity.

use std::path::PathBuf;

use scrapers::tx::counties::tx_clarity_results;

#[tokio::main]
async fn main() {
    let csv_path = std::env::args()
        .nth(1)
        .map(PathBuf::from);

    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    println!("=== TX Clarity County Results ===\n");
    println!("Data dir: {}\n", tx_clarity_results::clarity_data_path().display());

    match tx_clarity_results::run(&pool.connection, csv_path).await {
        Ok(()) => println!("\n✓ Done."),
        Err(e) => {
            eprintln!("\n✗ Error: {}", e);
            std::process::exit(1);
        }
    }
}
