//! Scrape Texas Republican Federal primary race results from the Civix Election Night Results page.
//!
//! Writes to ingest_staging.stg_tx_results_sos_civix (race, choice, party, early_votes, votes_for_candidate, total_votes, vote_pct).
//!
//! Requires chromedriver running: `chromedriver --port=9515`
//! Then: cargo run --bin tx_scrape_civix_fed_rep

use thirtyfour::prelude::*;

use scrapers::tx::tx_civix_fed_rep_results::{scrape_civix_one_election, write_results_to_db};

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        db::init_pool().await.map_err(|e| format!("DB pool: {}", e))?;
        let pool = db::pool().await;

        let mut caps = DesiredCapabilities::chrome();
        caps.add_arg("--no-sandbox").unwrap();
        caps.add_arg("--disable-dev-shm-usage").unwrap();
        caps.add_arg("--headless=new").unwrap();

        // Republican primary: one driver session so the initial modal appears.
        let driver = WebDriver::new("http://localhost:9515", caps.clone())
            .await
            .map_err(|e| format!("Connect to chromedriver at localhost:9515: {}. Run: chromedriver --port=9515", e))?;
        let rep_rows = scrape_civix_one_election(&driver, true).await?;
        driver.quit().await?;

        // Democratic primary: new driver session so the initial modal appears again.
        let driver = WebDriver::new("http://localhost:9515", caps)
            .await
            .map_err(|e| format!("Connect to chromedriver at localhost:9515: {}", e))?;
        let dem_rows = scrape_civix_one_election(&driver, false).await?;
        driver.quit().await?;

        let mut all_rows = rep_rows;
        all_rows.extend(dem_rows);

        let count = write_results_to_db(&pool.connection, &all_rows).await?;
        println!("Wrote {} rows to ingest_staging.stg_tx_results_sos_civix", count);
        Ok(())
    })
}
