use scrapers::mn::sos::fetch_results;

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    if let Err(err) = fetch_results().await {
        println!("error running example: {}", err);
    }
}
