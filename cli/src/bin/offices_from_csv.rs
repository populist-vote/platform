use std::{error::Error, io, process};

use db::{CreateOfficeInput, Office};

async fn create_offices() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(io::stdin());
    for result in rdr.deserialize() {
        let new_record_input: CreateOfficeInput = result?;
        let new_office = Office::create(&pool.connection, &new_record_input).await?;
        println!("Created new office record: {:?}", new_office.title);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = create_offices().await {
        println!("error running example: {}", err);
        process::exit(1);
    }
}
