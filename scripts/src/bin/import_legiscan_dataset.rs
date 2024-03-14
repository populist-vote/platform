use base64::{engine::general_purpose, Engine as _};
use clap::Parser;
use colored::*;
use db::models::enums::{BillStatus, PoliticalScope, State};
use db::{Bill, Chamber, UpsertBillInput};
use legiscan::GetBillResponse;
use slugify::slugify;
use std::collections::HashMap;
use std::error::Error;
use std::process;
use std::str::FromStr;
use std::time::Instant;

async fn import_legiscan_dataset(
    session_id: i32,
    state: State,
    year: i32,
) -> Result<(), Box<dyn Error>> {
    // Fetch dataset from Legiscan
    let legiscan = legiscan::LegiscanProxy::new().unwrap();
    let dataset_list = legiscan
        .get_dataset_list(
            Some(state.to_string().as_str()),
            Some(year.to_string().as_str()),
        )
        .await
        .unwrap();

    db::init_pool().await.unwrap();
    let db_pool = &db::pool().await.connection;

    let start = Instant::now();

    let session = sqlx::query!(
        r#"
            SELECT id, legiscan_dataset_hash
            FROM session
            WHERE legiscan_session_id = $1
        "#,
        session_id
    )
    .fetch_one(db_pool)
    .await;

    // Check hash on dataset to determine if we need to update the dataset
    let hash = dataset_list[0].clone().dataset_hash.clone();

    let populist_session_id;

    match session {
        Ok(session) => {
            populist_session_id = session.id;
            if session.legiscan_dataset_hash == Some(hash.clone()) {
                println!("\n\n🟢 Dataset already up to date.  No new bills found.\n");
                process::exit(0);
            } else {
                println!("\n\n🟡 Dataset has changed.  Updating dataset and importing new bills.\n")
            }
        }
        Err(_) => {
            println!("\n\n🔴 Error: Populist session not found: {}\n", session_id);
            process::exit(1);
        }
    }

    let access_key = dataset_list[0].access_key.clone();
    let dataset = legiscan.get_dataset(session_id, &access_key).await.unwrap();
    let zip = dataset.zip;
    // Decode base64 encoded zip file
    let zip_bytes = general_purpose::STANDARD.decode(zip.as_bytes()).unwrap();
    // Extract files from zip
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes)).unwrap();

    let mut bills_hash_map: HashMap<i32, legiscan::Bill> = HashMap::new();

    for i in 0..zip.len() {
        let file = zip.by_index(i).unwrap();
        // Extract all /bill/ files
        if file.name().contains("/bill/") {
            let json: GetBillResponse = serde_json::from_reader(file).unwrap();
            let bill = json.bill;
            bills_hash_map.insert(bill.bill_id, bill);
        }
    }

    let bill_ids = bills_hash_map.keys().map(|&k| k).collect::<Vec<i32>>();
    // Filter down the hashmap to bills that do not yet exist in the database
    let existing_bills = sqlx::query!(
        r#"
                SELECT legiscan_bill_id FROM bill WHERE legiscan_bill_id = ANY($1)
            "#,
        bill_ids.as_slice()
    )
    .fetch_all(db_pool)
    .await?;

    let new_bills = bills_hash_map
        .iter()
        .filter(|(k, _)| {
            !existing_bills
                .iter()
                .any(|x| x.legiscan_bill_id.unwrap() == **k)
        })
        .collect::<HashMap<_, _>>();

    println!(
        "\n\n📊 {} new bills found in dataset.  Importing bill data",
        new_bills.len().to_string().bright_green().bold()
    );

    for (_, bill) in new_bills.iter() {
        let input = UpsertBillInput {
            id: None,
            slug: Some(slugify!(&format!(
                "{}{}{}",
                &bill.state.clone(),
                &bill.bill_number,
                "-2023" // Need to make this dynamic, fetch session from db
            ))),
            title: Some(bill.title.clone()),
            populist_title: Some(bill.title.clone()),
            bill_number: bill.bill_number.clone(),
            status: match bill.status {
                1 => BillStatus::Introduced,
                2 => BillStatus::InConsideration,
                4 => BillStatus::BecameLaw,
                _ => BillStatus::Unknown,
            },
            description: None,
            session_id: populist_session_id,
            official_summary: None,
            populist_summary: None,
            full_text_url: Some(bill.state_link.clone()),
            legiscan_bill_id: Some(bill.bill_id),
            legiscan_session_id: Some(bill.session_id),
            legiscan_committee_id: None,
            legiscan_committee: None,
            legiscan_last_action: None,
            legiscan_last_action_date: None,
            history: Some(serde_json::to_value(bill.history.clone()).unwrap()),
            state: Some(State::from_str(&bill.state).unwrap()),
            legiscan_data: Some(serde_json::to_value(bill).unwrap()),
            votesmart_bill_id: None,
            arguments: None,
            political_scope: match bill.state.to_string().as_str() {
                "US" => Some(PoliticalScope::Federal),
                _ => Some(PoliticalScope::State),
            },
            bill_type: Some(bill.bill_type.clone()),
            chamber: match bill.current_body.to_string().as_str() {
                "H" => Some(Chamber::House),
                "S" => Some(Chamber::Senate),
                _ => None,
            },
            attributes: Some(serde_json::to_value("{}").unwrap()),
        };
        Bill::upsert(db_pool, &input).await.unwrap();
    }

    // Update legiscan_dataset_hash for session
    sqlx::query!(
        r#"
            UPDATE session
            SET legiscan_dataset_hash = $1
            WHERE id = $2
        "#,
        hash.clone(),
        populist_session_id
    )
    .execute(db_pool)
    .await?;

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
    #[arg(long)]
    session_id: i32,
    #[arg(long)]
    state: State,
    #[arg(long)]
    year: i32,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if let Err(err) = import_legiscan_dataset(args.session_id, args.state, args.year).await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
