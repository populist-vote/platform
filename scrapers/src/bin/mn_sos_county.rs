use scrapers::mn_sos_county::get_mn_sos_candidate_filings_county;

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    if let Err(err) = get_mn_sos_candidate_filings_county().await {
        println!("error running example: {}", err);
    }
}
