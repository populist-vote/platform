use scrapers::mn_sos_local::get_mn_sos_candidate_filings_local;

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    if let Err(err) = get_mn_sos_candidate_filings_local().await {
        println!("error running example: {}", err);
    }
}
