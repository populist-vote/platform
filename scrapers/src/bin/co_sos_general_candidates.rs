use scrapers::{Scraper, ScraperContext};

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;
    let context = ScraperContext { db: pool };
    let scraper = scrapers::co::sos::general_candidates::Scraper::default();
    if let Err(err) = scraper.run_local(&context).await {
        println!(
            "Error scraping data from CO SOS general candidate filings: {}",
            err
        );
    }
}
