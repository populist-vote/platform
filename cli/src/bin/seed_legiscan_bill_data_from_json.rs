use db::models::enums::LegislationStatus;
use legiscan::{Bill, BillStatus as LegiscanBillStatus};
use std::{env, error::Error, fs, io, path::Path, process};

async fn seed_bills() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    let args: Vec<String> = env::args().collect();

    let dir = Path::new(&args[1]);

    let mut count = 0;
    for entry in fs::read_dir(dir.to_owned())? {
        let path = entry?.path();
        let file = fs::File::open(path)?;
        let reader = io::BufReader::new(file);

        let json: serde_json::Value =
            serde_json::from_reader(reader).expect("JSON was improperly formatted");

        let json = &json["bill"];

        let bill: Bill =
            serde_json::from_value(json.to_owned()).expect("JSON did not fit bill prototype");

        let last_vote_chamber = match bill.votes.last() {
            Some(v) => v.chamber.to_owned(),
            None => "unknown".to_string(),
        };
        let legislation_status = match legiscan::BillStatus::try_from(bill.status)
            .unwrap_or(LegiscanBillStatus::NotIntroduced)
        {
            LegiscanBillStatus::NotIntroduced => LegislationStatus::Unknown,
            LegiscanBillStatus::Introduced => LegislationStatus::Introduced,
            LegiscanBillStatus::Engrossed => match last_vote_chamber.as_str() {
                "S" => LegislationStatus::PassedSenate,
                "H" => LegislationStatus::PassedHouse,
                _ => LegislationStatus::PassedHouse,
            },
            LegiscanBillStatus::Enrolled => LegislationStatus::SentToExecutive,
            LegiscanBillStatus::Passed => LegislationStatus::BecameLaw,
            LegiscanBillStatus::Vetoed => LegislationStatus::Vetoed,
            LegiscanBillStatus::Failed => match last_vote_chamber.as_str() {
                "S" => LegislationStatus::FailedSenate,
                "H" => LegislationStatus::FailedHouse,
                _ => LegislationStatus::PassedHouse,
            },
        };
        let input = db::CreateBillInput {
            slug: None,
            title: bill.title.to_owned(),
            legislation_status,
            bill_number: bill.bill_number.to_owned(),
            description: None,
            official_summary: None,
            populist_summary: None,
            full_text_url: Some(bill.url.to_owned()),
            legiscan_bill_id: Some(bill.bill_id),
            legiscan_data: Some(serde_json::to_value(bill.clone()).unwrap()),
            votesmart_bill_id: None,
            arguments: None,
        };
        db::Bill::create(&pool.connection, &input).await?;
        println!("Creating bill: {}", bill.bill_id.to_owned());
        count += 1;
    }
    println!("Created {} new bill records", count);
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = seed_bills().await {
        println!("Seeding bills from JSON directory: {}", err);
        process::exit(1);
    }
}

// cargo run --seed_bills seed_legiscan_bill_data_from_json < file_path
