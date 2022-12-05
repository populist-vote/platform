use colored::*;
use serde::Deserialize;
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::process;
use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
struct OrganizationIssueTag {
    organization_id: Uuid,
    issue_tag_id: Uuid,
}

async fn organization_issue_tags_from_csv() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let mut sp = Spinner::new(
        Spinners::Dots5,
        "Upserting organization issue tags from CSV".into(),
    );

    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    for result in rdr.deserialize() {
        let input: OrganizationIssueTag = result?;
        let _record = sqlx::query!(
            r#"
            INSERT INTO organization_issue_tags (organization_id, issue_tag_id) VALUES ($1, $2) 
            ON CONFLICT DO NOTHING
            RETURNING organization_id, issue_tag_id
        "#,
            input.organization_id,
            input.issue_tag_id
        )
        .fetch_one(&pool.connection)
        .await
        .expect(format!("Failed to insert race_candidate: {:?}", input).as_str());
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
    if let Err(err) = organization_issue_tags_from_csv().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
