use std::{error::Error, process};

async fn update_legiscan_bill_data() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    server::jobs::update_legiscan_bill_data::run()
        .await
        .map_err(|e| tracing::error!("Failed to update bill data: {}", e))
        .ok();
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = update_legiscan_bill_data().await {
        println!("error running example: {}", err);
        process::exit(1);
    }
}
