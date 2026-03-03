//! Merge ingest_staging.stg_tx_results_sos into production (race_candidates.votes, race totals).
//! Use --dry-run to report what would be updated without writing.
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
        println!("=== TX Results Merge (DRY RUN — no updates to race/race_candidates; unmatched rows still written to stg_tx_results_sos_unmatched) ===\n");
    } else if test_merge {
        println!("=== TX Results Merge (TEST — only office_name = \"U. S. Senator\") ===\n");
    } else {
        println!("=== TX Results Merge ===\n");
    }

    match tx_merge_results::merge_stg_tx_results_to_production(&pool.connection, dry_run, test_merge).await {
        Ok(stats) => {
            println!("Staging rows processed: {}", stats.staging_rows);
            println!("  Matched (ref_key found in race_candidates): {}", stats.matched);
            println!("  Unmatched (recorded in ingest_staging.stg_tx_results_sos_unmatched): {}", stats.unmatched);
            if dry_run {
                println!("\n[DRY RUN] Would have updated {} race_candidates and their races (skipped). Unmatched rows written to ingest_staging.stg_tx_results_sos_unmatched.", stats.matched);
            } else {
                println!("  race_candidates updated: {}", stats.race_candidates_updated);
                println!("  races updated: {}", stats.races_updated);
            }
            println!("\n✓ Merge completed successfully.");
        }
        Err(e) => {
            eprintln!("\n✗ Error: {}", e);
            std::process::exit(1);
        }
    }
}
