use scrapers::mn_sos_candidate_filings_fed_state_county::{
    get_mn_sos_candidate_filings_fed_state_county,
    get_mn_sos_candidate_filings_fed_state_county_primaries,
};
use scrapers::mn_sos_candidate_filings_local::{
    get_mn_sos_candidate_filings_local, get_mn_sos_candidate_filings_local_primaries,
};

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    if let Err(err) = get_mn_sos_candidate_filings_fed_state_county().await {
        println!("error scraping data from MN SOS candidate filings: {}", err);
    }
    if let Err(err) = get_mn_sos_candidate_filings_fed_state_county_primaries().await {
        println!("error scraping data from MN SOS candidate filings: {}", err);
    }
    if let Err(err) = get_mn_sos_candidate_filings_local().await {
        println!("error running example: {}", err);
    }
    if let Err(err) = get_mn_sos_candidate_filings_local_primaries().await {
        println!("error running example: {}", err);
    }
}
