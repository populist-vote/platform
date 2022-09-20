use colored::*;
use serde::Deserialize;
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::io;
use std::process;
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct RaceCandidate {
    race_id: Uuid,
    candidate_id: Uuid,
}

async fn upsert_race_candidates() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let mut sp = Spinner::new(
        Spinners::Dots5,
        "Upserting race_candidate records from CSV".into(),
    );

    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(io::stdin());
    for result in rdr.deserialize() {
        let input: RaceCandidate = result?;
        let _record = sqlx::query!(
            r#"
            INSERT INTO race_candidates (race_id, candidate_id) VALUES ($1, $2) 
            RETURNING race_id, candidate_id
        "#,
            input.race_id,
            input.candidate_id
        )
        .fetch_one(&pool.connection)
        .await
        .expect(format!("Failed to insert race_candidate: {:?}", input).as_str());
    }

    sp.stop();
    let duration = start.elapsed();
    eprintln!("✅ {}", "Success".bright_green().bold());
    eprintln!("🕑 {:?}", duration);
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = upsert_race_candidates().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
