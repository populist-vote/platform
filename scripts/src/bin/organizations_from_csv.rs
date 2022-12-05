use colored::*;
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::process;
use std::time::Instant;

async fn upsert_organizations_from_csv() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let mut sp = Spinner::new(Spinners::Dots5, "Upserting office records from CSV".into());
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    for result in rdr.deserialize() {
        let input: db::UpsertOrganizationInput = result?;

        let _organization = db::Organization::upsert(&pool.connection, &input)
            .await
            .expect(format!("Failed to upsert organization: {:?}", input.name).as_str());
    }

    sp.stop();
    let duration = start.elapsed();
    eprintln!(
        "
✅ {}",
        "Success".bright_green().bold()
    );
    eprintln!(
        "
🕑 {:?}",
        duration
    );
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = upsert_organizations_from_csv().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
