use scrapers::mn::sos::{
    get_mn_sos_candidate_filings_fed_state_county,
    get_mn_sos_candidate_filings_fed_state_county_primaries, get_mn_sos_candidate_filings_local,
    get_mn_sos_candidate_filings_local_primaries,
};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use thirtyfour::prelude::*;

async fn close_driver(driver: WebDriver) -> Result<(), Box<dyn std::error::Error>> {
    driver.quit().await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--no-sandbox").unwrap();
    caps.add_arg("--disable-dev-shm-usage").unwrap();
    caps.add_experimental_option("detach", true).unwrap();
    let driver = WebDriver::new("http://localhost:9515", caps).await.unwrap();

    db::init_pool().await.unwrap();

    // SCRAPE FED STATE COUNTY GENERAL DATA FROM SOS SITE
    // if let Err(err) = get_mn_sos_candidate_filings_fed_state_county(&driver).await {
    //     println!("error scraping data from MN SOS candidate filings: {}", err);
    // }

    // SCRAPE FED STATE COUNTY PRIMARY DATA FROM SOS SITE
    // if let Err(err) = get_mn_sos_candidate_filings_fed_state_county_primaries(&driver).await {
    //     println!("error scraping data from MN SOS candidate filings: {}", err);
    // }

    // SCRAPE LOCAL GENERAL DATA FROM SOS SITE
    if let Err(err) = get_mn_sos_candidate_filings_local(&driver).await {
        println!("error running example: {}", err);
    }

    // SCRAPE LOCAL PRIMARY DATA FROM SOS SITE
    // if let Err(err) = get_mn_sos_candidate_filings_local_primaries(&driver).await {
    //     println!("error running example: {}", err);
    // }

    println!("Press Enter to close the browser window...");
    io::stdout().flush().unwrap();

    // Keep the program running until Enter is pressed
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    // Close the driver when Enter is pressed
    if let Err(e) = close_driver(driver).await {
        println!("Error closing browser: {}", e);
    }
}
