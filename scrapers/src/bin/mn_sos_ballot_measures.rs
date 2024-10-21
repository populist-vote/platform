use scrapers::{util::run_with_timer, Scraper, ScraperContext};

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;
    let context = ScraperContext { db: pool };
    let scraper = scrapers::mn::sos::ballot_measures::Scraper::default();
    if let Err(err) = run_with_timer("Scraping data from MN SOS".into(), || async {
        scraper.run_local(&context).await?;
        Ok(())
    })
    .await
    {
        println!("Error scraping data from MN SOS ballot measures: {}", err);
    }
}
