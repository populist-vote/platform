use std::{error::Error, io, process};

use db::{Office, UpsertOfficeInput};

async fn create_offices() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(io::stdin());
    for result in rdr.deserialize() {
        let input: UpsertOfficeInput = result?;
        let office = Office::upsert(&pool.connection, &input)
            .await
            .expect(format!("Failed to upsert office: {:?}", input.slug).as_str());
        println!("Created new office record: {:?}", office.title);
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
