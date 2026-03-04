//! Merge TX Other results from staging into production (ingest_staging.stg_tx_results_other).
//! Updates race_candidates.votes and race totals. Use --dry-run to report without writing.
//! Use --test-merge to only merge rows where office_name = "U. S. Senator".

use scrapers::processors::tx::tx_merge_results;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let test_merge = args.iter().any(|a| a == "--test-merge");

    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    if dry_run {
        println!("=== TX Other Results Merge (DRY RUN — no updates; unmatched rows still written to stg_tx_results_other_unmatched) ===\n");
    } else if test_merge {
        println!("=== TX Other Results Merge (TEST — only office_name = \"U. S. Senator\") ===\n");
    } else {
        println!("=== TX Other Results Merge ===\n");
    }

    println!("--- Other (ingest_staging.stg_tx_results_other) ---");
    match tx_merge_results::merge_stg_tx_results_other_to_production(&pool.connection, dry_run, test_merge).await {
        Ok(stats) => {
            println!("  Staging rows processed: {}", stats.staging_rows);
            println!("  Matched: {}", stats.matched);
            println!("  Unmatched (stg_tx_results_other_unmatched): {}", stats.unmatched);
            if !dry_run {
                println!("  race_candidates updated: {}", stats.race_candidates_updated);
                println!("  races updated: {}", stats.races_updated);
            }
            println!();
        }
        Err(e) => {
            eprintln!("\n✗ Other merge error: {}", e);
            std::process::exit(1);
        }
    }

    println!("✓ TX Other results merge completed.");
}
