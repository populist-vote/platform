use colored::*;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::process;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RaceCandidateResult {
    // race_id: uuid::Uuid,
    candidate_id: uuid::Uuid,
    // votes: i32,
    #[serde(rename = "total votes")]
    total_votes: i32,
}

async fn import_race_results_from_csv() -> Result<(), Box<dyn Error>> {
    let mut sp = Spinner::new(Spinners::Dots5, "Importing race results from CSV".into());
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    let mut total_counts = HashMap::new();

    let mut rdr = csv::Reader::from_reader(io::stdin());
    for result in rdr.deserialize() {
        let candidate: RaceCandidateResult = result?;
        let record = sqlx::query!(
            r#"
            UPDATE
                race_candidates
            SET
                votes = $1
            WHERE
                candidate_id = $2
            RETURNING race_id
        "#,
            candidate.total_votes,
            candidate.candidate_id
        )
        .fetch_optional(&pool.connection)
        .await?;

        if let Some(record) = record {
            if let Some(count) = total_counts.get_mut(&record.race_id) {
                *count += candidate.total_votes;
            } else {
                total_counts.insert(record.race_id.clone(), candidate.total_votes);
            }
        }
    }

    for (race_id, count) in &total_counts {
        let _query = sqlx::query!(
            r#"
            UPDATE race
            SET total_votes = $1,
                winner_id = (SELECT candidate_id FROM race_candidates WHERE race_id = $2 ORDER BY votes DESC LIMIT 1)
            WHERE id = $2
            "#,
            count,
            race_id
        )
        .fetch_optional(&pool.connection)
        .await;
    }

    sp.stop();
    eprintln!("\n✅ {}", "Success".bright_green().bold());
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = import_race_results_from_csv().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
