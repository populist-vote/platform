use std::{thread::sleep, time::Duration};

use scrapers::{util::run_with_timer, Scraper, ScraperContext};

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;
    let context = ScraperContext { db: pool };
    let scraper = scrapers::co::sos::general_candidates::Scraper::default();
    if let Err(err) = run_with_timer("Scraping data from CO SOS".into(), || async {
        scraper.run_local(&context).await?;

        Ok(())
    })
    .await
    {
        println!(
            "Error scraping data from CO SOS general candidate filings: {}",
            err
        );
    }
}
