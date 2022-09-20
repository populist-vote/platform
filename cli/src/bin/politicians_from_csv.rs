use colored::*;
use db::UpsertPoliticianInput;
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::io;
use std::process;
use std::time::Instant;

async fn upsert_politicians_from_csv() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let mut sp = Spinner::new(
        Spinners::Dots5,
        "Upserting politician records from CSV".into(),
    );
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(io::stdin());
    for result in rdr.deserialize() {
        let input: UpsertPoliticianInput = result?;

        let _politician = db::Politician::upsert(&pool.connection, &input)
            .await
            .expect(
                format!(
                    "Failed to upsert politician: {:?} {:?}",
                    input.first_name, input.last_name
                )
                .as_str(),
            );

        sp.stop();
        let duration = start.elapsed();
        eprintln!("\n✅ {}\n", "Success".bright_green().bold());
        eprintln!("🕑 {:?}", duration);
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = upsert_politicians_from_csv().await {
        println!("error upserting politicians: {}", err);
        process::exit(1);
    }
}
