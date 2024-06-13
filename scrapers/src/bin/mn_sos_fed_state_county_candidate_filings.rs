use scrapers::mn_sos_fed_state_county_candidate_filings::{
    get_mn_sos_candidate_filings_fed_state_county,
    get_mn_sos_candidate_filings_primary_fed_state_county,
};

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    if let Err(err) = get_mn_sos_candidate_filings_fed_state_county().await {
        println!("error scraping data from MN SOS candidate filings: {}", err);
    }
    if let Err(err) = get_mn_sos_candidate_filings_primary_fed_state_county().await {
        println!("error scraping data from MN SOS candidate filings: {}", err);
    }
}
