use std::error::Error;
use std::process;

use db::CreateRaceInput;

async fn seed_2020_races() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    // let us_senate_republican_primary = CreateRaceInput {
    //     slug: "us-senate-republican-primary-2020-colorado",
    //     title: "U.S. Senate Republican Primary",
    //     office_position: "U.S. Senate",
    //     office_id: None,
    //     official_website: None,
    //     race_type: db::models::enums::RaceType::Primary,
    //     state: db::models::enums::State::CO,
    // }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = seed_2020_races().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
