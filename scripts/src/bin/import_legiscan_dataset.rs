use base64::{engine::general_purpose, Engine as _};
use clap::Parser;
use colored::*;
use db::models::enums::{BillStatus, PoliticalScope, State};
use db::{Bill, Chamber, UpsertBillInput};
use legiscan::GetBillResponse;
use slugify::slugify;
use std::error::Error;
use std::process;
use std::str::FromStr;
use std::time::Instant;

async fn import_legiscan_dataset(session_id: i32) -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    let db_pool = &db::pool().await.connection;
    let start = Instant::now();

    // Fetch dataset from Legiscan
    let legiscan = legiscan::LegiscanProxy::new().unwrap();
    let session = sqlx::query!(
        r#"
            SELECT id
            FROM session
            WHERE legiscan_session_id = $1
        "#,
        session_id
    )
    .fetch_one(db_pool)
    .await;

    if session.is_err() {
        println!("\n\n🔴 Error: Populist session not found: {}\n", session_id);
        process::exit(1);
    }

    let dataset_list = legiscan
        .get_dataset_list(Some("MN"), Some("2024"))
        .await
        .unwrap();
    let access_key = dataset_list[0].access_key.clone();
    let dataset = legiscan.get_dataset(session_id, &access_key).await.unwrap();
    let zip = dataset.zip;
    // Decode base64 encoded zip file
    let zip_bytes = general_purpose::STANDARD.decode(zip.as_bytes()).unwrap();
    // Extract files from zip
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes)).unwrap();
    for i in 0..zip.len() {
        let file = zip.by_index(i).unwrap();
        // Extract all /bill/ files
        if file.name().contains("/bill/") {
            let json: GetBillResponse = serde_json::from_reader(file).unwrap();
            let bill = json.bill;

            let existing_bill = sqlx::query!(
                r#"
                    SELECT id
                    FROM bill
                    WHERE legiscan_bill_id = $1
                "#,
                bill.bill_id
            )
            .fetch_optional(db_pool)
            .await?;

            let mut input = UpsertBillInput {
                id: None,
                slug: Some(slugify!(&format!(
                    "{}{}{}",
                    &bill.clone().state,
                    &bill.clone().bill_number,
                    "2023-2024" // Need to make this dynamic, fetch session from db
                ))),
                title: Some(bill.clone().title),
                bill_number: bill.clone().bill_number,
                status: match bill.clone().status {
                    1 => BillStatus::Introduced,
                    2 => BillStatus::InConsideration,
                    4 => BillStatus::BecameLaw,
                    _ => BillStatus::Unknown,
                },
                description: None,
                session_id: session.as_ref().unwrap().id,
                official_summary: None,
                populist_summary: None,
                full_text_url: Some(bill.clone().state_link),
                legiscan_bill_id: Some(bill.clone().bill_id),
                legiscan_session_id: Some(bill.clone().session_id),
                legiscan_committee_id: None,
                legiscan_committee: None,
                legiscan_last_action: None,
                legiscan_last_action_date: None,
                history: Some(serde_json::to_value(bill.clone().history).unwrap()),
                state: Some(State::from_str(&bill.clone().state).unwrap()),
                legiscan_data: Some(serde_json::to_value(bill.clone()).unwrap()),
                votesmart_bill_id: None,
                arguments: None,
                political_scope: match bill.clone().state.to_string().as_str() {
                    "US" => Some(PoliticalScope::Federal),
                    _ => Some(PoliticalScope::State),
                },
                bill_type: Some(bill.clone().bill_type),
                chamber: match bill.clone().current_body.to_string().as_str() {
                    "H" => Some(Chamber::House),
                    "S" => Some(Chamber::Senate),
                    _ => None,
                },
                attributes: Some(serde_json::to_value("{}").unwrap()),
            };

            if let Some(existing_bill) = existing_bill {
                input.id = Some(existing_bill.id);
            }

            Bill::upsert(db_pool, &input).await.unwrap();
        }
    }

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

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    session_id: i32,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Err(err) = import_legiscan_dataset(args.session_id).await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
