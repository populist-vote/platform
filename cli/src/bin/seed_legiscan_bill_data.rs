use std::error::Error;
use std::process;

use db::models::enums::LegislationStatus;
use legiscan::BillStatus as LegiscanBillStatus;

async fn seed_bills() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    let proxy = legiscan::LegiscanProxy::new().unwrap();
    let masterlist = proxy.get_master_list_by_state("CO").await.unwrap();
    let bill_ids: Vec<i32> = masterlist.iter().map(|bill| bill.bill_id).collect();

    for id in bill_ids.iter() {
        let bill = proxy.get_bill(id.to_owned()).await.unwrap();
        let last_vote_chamber = match bill.votes.last() {
            Some(v) => v.chamber.to_owned(),
            None => "unknown".to_string(),
        };
        let legislation_status = match legiscan::BillStatus::try_from(bill.status).unwrap() {
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
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = seed_bills().await {
        println!("Error seeding bills: {}", err);
        process::exit(1);
    }
}
