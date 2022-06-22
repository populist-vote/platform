use std::{error::Error, io, process};

use db::{CreateRaceInput, Race};

async fn create_races() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(io::stdin());
    for result in rdr.deserialize() {
        let new_record_input: CreateRaceInput = result?;
        let new_race = Race::create(&pool.connection, &new_record_input).await?;
        println!("New race created: = {:?}", new_race.title);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = create_races().await {
        println!("error running example: {}", err);
        process::exit(1);
    }
}
