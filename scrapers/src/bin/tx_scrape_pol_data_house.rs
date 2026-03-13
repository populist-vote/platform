//! TX U.S. House candidate web scrape: loads TX candidates (March 3 primary),
//! discovers campaign/official URLs via DuckDuckGo, scrapes pages for social links,
//! email, profile image; writes to ingest_staging.stg_tx_scraped_us_house_candidates.
//! Plan: docs/tx_house_candidate_web_scrape_plan.md
//!
//! Usage: tx_house_scrape_web_contacts [LIMIT]
//!   LIMIT  optional; max number of candidates to scrape (e.g. 5 for a quick test). If omitted, all candidates are scraped.

use scrapers::tx::tx_candidate_web_scrape;

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    let limit = std::env::args()
        .nth(1)
        .and_then(|s| s.parse::<usize>().ok());

    println!("=== TX U.S. House Candidate Web Scrape ===\n");
    println!("Election: March 3 primary (0d586931-c119-4fe7-814f-f679e91282a8)");
    println!("Staging: ingest_staging.stg_tx_scraped_us_house_candidates");
    if let Some(n) = limit {
        println!("Limit: {} candidates", n);
    }
    println!();

    match tx_candidate_web_scrape::run(&pool.connection, limit).await {
        Ok(()) => {
            println!("\n✓ Scrape completed successfully.");
            println!("Review staging: SELECT * FROM ingest_staging.stg_tx_scraped_us_house_candidates;");
        }
        Err(e) => {
            eprintln!("\n✗ Scrape failed: {}", e);
            std::process::exit(1);
        }
    }
}
