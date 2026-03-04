//! Process Hart/CIRA PDFs from data/tx/counties/hart/input, write PDF-style CSVs
//! to data/tx/counties/hart/output, and load rows into ingest_staging.stg_tx_results_hart.

use scrapers::tx::counties::tx_hart_results;

#[tokio::main]
async fn main() {
    println!("=== TX Hart County Results ===\n");
    println!("Input:  {}", tx_hart_results::hart_input_path().display());
    println!("Output: {}\n", tx_hart_results::hart_output_path().display());

    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    match tx_hart_results::run(&pool.connection).await {
        Ok(()) => println!("\n✓ Done."),
        Err(e) => {
            eprintln!("\n✗ Error: {}", e);
            std::process::exit(1);
        }
    }
}
