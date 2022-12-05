use colored::*;
use serde::Deserialize;
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::process;
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct OrganizationEndorsements {
    politician_id: Uuid,
    organization_endorsement_ids: String,
}

async fn upsert_organization_endorsements() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let mut sp = Spinner::new(
        Spinners::Dots5,
        "Upserting politician organization endorsements records from CSV".into(),
    );
    db::init_pool().await.unwrap();
    let pool = db::pool().await;
    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    for result in rdr.deserialize() {
        let input: OrganizationEndorsements = result?;

        if input.organization_endorsement_ids.is_empty() {
            continue;
        }

        let politician_endorsement_ids: Vec<Uuid> = input
            .organization_endorsement_ids
            .replace(" ", "")
            .split(',')
            .map(|id| Uuid::parse_str(id).unwrap())
            .collect();

        let _record = sqlx::query!(
            r#"
            INSERT INTO politician_organization_endorsements (politician_id, organization_id) VALUES ($1, UNNEST($2::uuid[]))
            ON CONFLICT (politician_id, organization_id) DO NOTHING
            RETURNING politician_id, organization_id
        "#,
            input.politician_id,
            &politician_endorsement_ids
        )
        .fetch_optional(&pool.connection)
        .await
        .expect(format!("Failed to insert politician_organization_endorsements: {:?}", input).as_str());
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
    if let Err(err) = upsert_organization_endorsements().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
