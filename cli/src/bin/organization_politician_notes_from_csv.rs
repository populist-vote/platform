use colored::*;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::process;
use std::time::Instant;

#[derive(Debug, Serialize, Deserialize)]
struct OrganizationPoliticianNote {
    organization_id: uuid::Uuid,
    politician_id: uuid::Uuid,
    election_id: uuid::Uuid,
    issue_tag_ids: Vec<uuid::Uuid>,
    note_en: String,
    note_es: String,
    note_so: String,
    note_hmn: String,
}

async fn organization_politician_notes_from_csv() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let mut sp = Spinner::new(Spinners::Dots5, "Upserting office records from CSV".into());

    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    for result in rdr.deserialize() {
        let input: OrganizationPoliticianNote = result?;

        let _input = sqlx::query!(
            r#"
            INSERT INTO organization_politician_notes (
                organization_id,
                politician_id,
                election_id,
                issue_tag_ids,
                notes
            )
            VALUES (
                $1,
                $2,
                $3,
                $4,
                $5
            )
        "#,
            input.organization_id,
            input.politician_id,
            input.election_id,
            &input.issue_tag_ids,
            serde_json::json!({
                "en": input.note_en,
                "es": input.note_es,
                "so": input.note_so,
                "hmn": input.note_hmn,
            })
        ).execute(&pool.connection)
        .await
        .expect(
            format!(
                "Something went wrong inserting organization politician note for organization_id: {}, politician_id: {}, election_id: {}",
                input.organization_id,
                input.politician_id,
                input.election_id
            )
            .as_str(),
        );
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
    if let Err(err) = organization_politician_notes_from_csv().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
