use colored::*;
use serde::Deserialize;
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::process;
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct PoliticianEndorsments {
    politician_id: Uuid,
    politician_endorsement_ids: String,
}

async fn upsert_politician_endorsements() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let mut sp = Spinner::new(
        Spinners::Dots5,
        "Upserting politician politician endorsements records from CSV".into(),
    );
    db::init_pool().await.unwrap();
    let pool = db::pool().await;
    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    for result in rdr.deserialize() {
        let input: PoliticianEndorsments = result?;

        if input.politician_endorsement_ids.is_empty() {
            continue;
        }

        let politician_endorsement_ids: Vec<Uuid> = input
            .politician_endorsement_ids
            .replace(" ", "")
            .split(',')
            .map(|id| Uuid::parse_str(id).unwrap())
            .collect();

        let _record = sqlx::query!(
            r#"
            INSERT INTO politician_politician_endorsements (politician_id, politician_endorsement_id) VALUES ($1, UNNEST($2::uuid[]))
            ON CONFLICT (politician_id, politician_endorsement_id) DO NOTHING
            RETURNING politician_id, politician_endorsement_id
        "#,
            input.politician_id,
            &politician_endorsement_ids
        )
        .fetch_optional(&pool.connection)
        .await
        .expect(format!("Failed to insert politician_endorsements: {:?}", input).as_str());
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
    if let Err(err) = upsert_politician_endorsements().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
