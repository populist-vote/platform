//! Merge TX results from staging into production.
//! Sources: SOS (stg_tx_results_sos), Clarity (stg_tx_results_clarity), Hart (stg_tx_results_hart), Other (stg_tx_results_other), Civix (stg_tx_results_sos_civix).
//! Updates race_candidates.votes and race totals. Use --dry-run to report without writing.
//! Use --test-merge to only merge rows where office_name = "U. S. Senator" (or race ILIKE '%U. S. Senator%' for Civix).
//!
//! Usage: tx_merge_results [--sos] [--clarity] [--hart] [--other] [--civix]
//!   If no source flag is given, all five sources are merged.
//!   --dry-run   report without writing
//!   --test-merge   only merge rows where office_name = "U. S. Senator" (or race for Civix)

use scrapers::mergers::tx::tx_results;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let test_merge = args.iter().any(|a| a == "--test-merge");
    let do_sos = args.iter().any(|a| a == "--sos");
    let do_clarity = args.iter().any(|a| a == "--clarity");
    let do_hart = args.iter().any(|a| a == "--hart");
    let do_other = args.iter().any(|a| a == "--other");
    let do_civix = args.iter().any(|a| a == "--civix");
    let any_source = do_sos || do_clarity || do_hart || do_other || do_civix;
    let run_all = !any_source;

    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    if dry_run {
        println!("=== TX Results Merge (DRY RUN — no updates; unmatched rows still written to *_unmatched tables) ===\n");
    } else if test_merge {
        println!("=== TX Results Merge (TEST — only office_name = \"U. S. Senator\") ===\n");
    } else {
        println!("=== TX Results Merge ===\n");
    }

    let mut ran_any = false;

    if run_all || do_sos {
        ran_any = true;
        println!("--- SOS (ingest_staging.stg_tx_results_sos) ---");
        match tx_results::merge_stg_tx_results_sos_to_production(&pool.connection, dry_run, test_merge).await {
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
    }

    if run_all || do_clarity {
        ran_any = true;
        println!("--- Clarity (ingest_staging.stg_tx_results_clarity) ---");
        match tx_results::merge_stg_tx_results_clarity_to_production(&pool.connection, dry_run, test_merge).await {
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
    }

    if run_all || do_hart {
        ran_any = true;
        println!("--- Hart (ingest_staging.stg_tx_results_hart) ---");
        match tx_results::merge_stg_tx_results_hart_to_production(&pool.connection, dry_run, test_merge).await {
            Ok(stats) => {
                println!("  Staging rows processed: {}", stats.staging_rows);
                println!("  Matched: {}", stats.matched);
                println!("  Unmatched (stg_tx_results_hart_unmatched): {}", stats.unmatched);
                if !dry_run {
                    println!("  race_candidates updated: {}", stats.race_candidates_updated);
                    println!("  races updated: {}", stats.races_updated);
                }
                println!();
            }
            Err(e) => {
                eprintln!("\n✗ Hart merge error: {}", e);
                std::process::exit(1);
            }
        }
    }

    if run_all || do_other {
        ran_any = true;
        println!("--- Other (ingest_staging.stg_tx_results_other) ---");
        match tx_results::merge_stg_tx_results_other_to_production(&pool.connection, dry_run, test_merge).await {
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
    }

    if run_all || do_civix {
        ran_any = true;
        println!("--- Civix (ingest_staging.stg_tx_results_sos_civix) ---");
        match tx_results::merge_stg_tx_results_sos_civix_to_production(&pool.connection, dry_run, test_merge).await {
            Ok(stats) => {
                println!("  Staging rows processed: {}", stats.staging_rows);
                println!("  Matched: {}", stats.matched);
                println!("  Unmatched (stg_tx_results_sos_civix_unmatched): {}", stats.unmatched);
                if !dry_run {
                    println!("  race_candidates updated: {}", stats.race_candidates_updated);
                    println!("  races updated: {}", stats.races_updated);
                }
                println!();
            }
            Err(e) => {
                eprintln!("\n✗ Civix merge error: {}", e);
                std::process::exit(1);
            }
        }
    }

    if !ran_any {
        eprintln!("No sources selected. Use --sos, --clarity, --hart, --other, and/or --civix, or omit all to run every source.");
        std::process::exit(1);
    }

    println!("✓ TX results merge completed.");
}
