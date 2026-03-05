//! Merge TX results from staging into production: SOS (stg_tx_results_sos) and Clarity (stg_tx_results_clarity).
//! Updates race_candidates.votes and race (total_votes, num_precincts_reporting, total_precincts from staging). Use --dry-run to report without writing.
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
        println!("=== TX Results Merge (DRY RUN — no updates; unmatched rows still written to *_unmatched tables) ===\n");
    } else if test_merge {
        println!("=== TX Results Merge (TEST — only office_name = \"U. S. Senator\") ===\n");
    } else {
        println!("=== TX Results Merge ===\n");
    }

    // SOS
    println!("--- SOS (ingest_staging.stg_tx_results_sos) ---");
    match tx_merge_results::merge_stg_tx_results_sos_to_production(&pool.connection, dry_run, test_merge).await {
        Ok(stats) => {
            println!("  Staging rows processed: {}", stats.staging_rows);
            println!("  Matched: {}", stats.matched);
            println!("  Unmatched (stg_tx_results_sos_unmatched): {}", stats.unmatched);
            if !dry_run {
                println!("  race_candidates updated: {}", stats.race_candidates_updated);
                println!("  races updated: {}", stats.races_updated);
            }
            println!();
        }
        Err(e) => {
            eprintln!("\n✗ SOS merge error: {}", e);
            std::process::exit(1);
        }
    }

    // Clarity
    println!("--- Clarity (ingest_staging.stg_tx_results_clarity) ---");
    match tx_merge_results::merge_stg_tx_results_clarity_to_production(&pool.connection, dry_run, test_merge).await {
        Ok(stats) => {
            println!("  Staging rows processed: {}", stats.staging_rows);
            println!("  Matched: {}", stats.matched);
            println!("  Unmatched (stg_tx_results_clarity_unmatched): {}", stats.unmatched);
            if !dry_run {
                println!("  race_candidates updated: {}", stats.race_candidates_updated);
                println!("  races updated: {}", stats.races_updated);
            }
            println!();
        }
        Err(e) => {
            eprintln!("\n✗ Clarity merge error: {}", e);
            std::process::exit(1);
        }
    }

    println!("✓ TX results merge completed (SOS + Clarity).");
}
