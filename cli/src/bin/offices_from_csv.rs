use colored::*;
use db::{Office, UpsertOfficeInput};
use spinners::{Spinner, Spinners};
use std::time::Instant;
use std::{error::Error, io, process};

async fn create_offices() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let mut sp = Spinner::new(Spinners::Dots5, "Upserting office records from CSV".into());
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(io::stdin());
    for result in rdr.deserialize() {
        let input: UpsertOfficeInput = result?;
        let office = Office::upsert(&pool.connection, &input)
            .await
            .expect(format!("Failed to upsert office: {:?}", input.slug).as_str());
    }

    sp.stop();
    let duration = start.elapsed();
    eprintln!("\n✅ {}\n", "Success".bright_green().bold());
    eprintln!("🕑 {:?}", duration);

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = create_offices().await {
        println!("error running example: {}", err);
        process::exit(1);
    }
}
