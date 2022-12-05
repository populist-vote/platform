use chrono::NaiveDate;
use colored::*;
use serde::de::{Error as SerdeError, Unexpected};
use serde::Deserialize;
use serde::Deserializer;
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
    #[serde(deserialize_with = "bool_from_str")]
    is_running: bool,
    date_announced: Option<NaiveDate>,
    date_qualified: Option<NaiveDate>,
    date_dropped: Option<NaiveDate>,
    reason_dropped: Option<String>,
    qualification_method: Option<String>,
    qualification_info: Option<String>,
    votes: Option<i32>,
}

fn bool_from_str<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer)?.as_ref() {
        "TRUE" => Ok(true),
        "FALSE" => Ok(false),
        other => Err(SerdeError::invalid_value(
            Unexpected::Str(other),
            &"TRUE or FALSE",
        )),
    }
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
            INSERT INTO race_candidates (race_id, candidate_id, is_running, date_announced, date_qualified, date_dropped, reason_dropped, qualification_method, qualification_info, votes) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) 
            ON CONFLICT (race_id, candidate_id) DO UPDATE SET 
                is_running = EXCLUDED.is_running,
                date_announced = EXCLUDED.date_announced,
                date_qualified = EXCLUDED.date_qualified,
                date_dropped = EXCLUDED.date_dropped,
                reason_dropped = EXCLUDED.reason_dropped,
                qualification_method = EXCLUDED.qualification_method,
                qualification_info = EXCLUDED.qualification_info,
                votes = EXCLUDED.votes
            RETURNING *
        "#,
            input.race_id,
            input.candidate_id,
            input.is_running,
            input.date_announced,
            input.date_qualified,
            input.date_dropped,
            input.reason_dropped,
            input.qualification_method,
            input.qualification_info,
            input.votes
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
