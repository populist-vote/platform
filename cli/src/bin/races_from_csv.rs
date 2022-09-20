use colored::*;
use db::{Race, UpsertRaceInput};
use spinners::{Spinner, Spinners};
use std::time::Instant;
use std::{error::Error, io, process};

async fn upsert_races() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let mut sp = Spinner::new(Spinners::Dots5, "Upserting race records from CSV".into());
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(io::stdin());
    for result in rdr.deserialize() {
        let input: UpsertRaceInput = result?;
        let _race = Race::upsert(&pool.connection, &input)
            .await
            .expect(format!("Failed to upsert race: {:?}", input.slug).as_str());
    }

    sp.stop();
    let duration = start.elapsed();
    eprintln!("\n✅ {}\n", "Success".bright_green().bold());
    eprintln!("🕑 {:?}", duration);

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = upsert_races().await {
        println!("error running example: {}", err);
        process::exit(1);
    }
}
