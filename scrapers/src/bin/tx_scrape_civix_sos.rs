//! Scrape Texas SOS Civix Election Night Results.
//!
//! Writes to ingest_staging.stg_tx_results_sos_civix (race, choice, party, early_votes, votes_for_candidate, total_votes, vote_pct).
//!
//! Flags (exactly one required):
//!   --primary   Republican + Democratic 2026 primaries (two Chrome sessions).
//!   --general   2026 November General Election (one session; no Statewide Propositions tab).
//!
//! Requires chromedriver running: `chromedriver --port=9515`
//! Then: cargo run --bin tx_scrape_civix_sos -- --primary
//!       cargo run --bin tx_scrape_civix_sos -- --general

use thirtyfour::prelude::*;
use thirtyfour::ChromeCapabilities;

use scrapers::tx::tx_civix_sos_results::{
    scrape_civix_one_election, write_results_to_db, CivixElection, ResultRow,
};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();
    let run_primary = args.iter().any(|a| a == "--primary");
    let run_general = args.iter().any(|a| a == "--general");
    if run_primary == run_general {
        eprintln!("Specify exactly one of --primary or --general");
        eprintln!("  cargo run --bin tx_scrape_civix_sos -- --primary");
        eprintln!("  cargo run --bin tx_scrape_civix_sos -- --general");
        std::process::exit(1);
    }

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        db::init_pool()
            .await
            .map_err(|e| format!("DB pool: {}", e))?;
        let pool = db::pool().await;

        let mut caps = DesiredCapabilities::chrome();
        caps.add_arg("--no-sandbox").unwrap();
        caps.add_arg("--disable-dev-shm-usage").unwrap();
        // caps.add_arg("--headless=new").unwrap();

        let mut all_rows: Vec<ResultRow> = Vec::new();

        if run_primary {
            all_rows.extend(scrape_session(caps.clone(), CivixElection::RepublicanPrimary).await?);
            all_rows.extend(scrape_session(caps.clone(), CivixElection::DemocraticPrimary).await?);
        }

        if run_general {
            all_rows.extend(scrape_session(caps, CivixElection::NovemberGeneral).await?);
        }

        let count = write_results_to_db(&pool.connection, &all_rows).await?;
        println!(
            "Wrote {} rows to ingest_staging.stg_tx_results_sos_civix",
            count
        );
        Ok(())
    })
}
async fn scrape_session(
    caps: ChromeCapabilities,
    election: CivixElection,
) -> Result<Vec<ResultRow>, Box<dyn std::error::Error + Send + Sync>> {
    let driver = WebDriver::new("http://localhost:9515", caps)
        .await
        .map_err(|e| {
            format!(
                "Connect to chromedriver at localhost:9515: {}. Run: chromedriver --port=9515",
                e
            )
        })?;
    let rows = scrape_civix_one_election(&driver, election).await;
    driver.quit().await?;
    rows
}

