use colored::*;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::process;
use std::time::Instant;

#[derive(Debug, Deserialize, Serialize)]
struct LegiscanDocument {
    bill_id: i32,
    document_id: i32,
    document_type: String,
    document_size: i32,
    document_mime: String,
    document_desc: String,
    url: String,
    state_link: String,
}

async fn get_pdf_urls(file_path: PathBuf) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    // Initialize DB pool
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    // Open the file
    let file = File::open(&file_path)?;

    let mut sp = Spinner::new(Spinners::Dots5, "Processing CSV records...".into());

    let mut rdr = csv::Reader::from_reader(file);
    let mut record_count = 0;

    for result in rdr.deserialize() {
        sp.stop();

        let input: LegiscanDocument = result?;

        // Insert or update the record using sqlx
        match sqlx::query!(
            r#"
                UPDATE bill
                SET pdf_url = $1
                WHERE legiscan_bill_id = $2
            "#,
            input.state_link,
            input.bill_id
        )
        .execute(&pool.connection)
        .await
        {
            Ok(_) => eprintln!("Inserted/Updated document_id: {}", input.document_id),
            Err(e) => eprintln!("Error inserting document_id {}: {}", input.document_id, e),
        }

        record_count += 1;
        sp = Spinner::new(
            Spinners::Dots5,
            format!("Processed {} records...", record_count).into(),
        );
    }

    sp.stop();

    let duration = start.elapsed();
    eprintln!("\n✅ {}\n", "Success".bright_green().bold());
    eprintln!("🕑 {:?}", duration);
    eprintln!("Processed {} records", record_count);

    Ok(())
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <csv_file_path>", args[0]);
        process::exit(1);
    }

    let file_path = PathBuf::from(&args[1]);

    if let Err(err) = get_pdf_urls(file_path).await {
        eprintln!("Error running get_pdf_urls: {}", err);
        process::exit(1);
    }
}
