//! Texas SOS results processor: reads XML files from scrapers/data/tx/sos
//! and loads into ingest_staging.stg_tx_results_sos.

use scrapers::processors::tx::tx_results::process_tx_sos_results;

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    println!("=== TX SOS Results Processor ===\n");
    println!("Reading XML from data/tx/sos...\n");

    match process_tx_sos_results(&pool.connection, true).await {
        Ok((files, rows)) => {
            println!("\n✓ Processed {} file(s), {} rows loaded into ingest_staging.stg_tx_results_sos", files, rows);
            println!("\n  SELECT * FROM ingest_staging.stg_tx_results_sos;");
        }
        Err(e) => {
            eprintln!("\n✗ Error: {}", e);
            std::process::exit(1);
        }
    }
}
