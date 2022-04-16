use std::error::Error;
use std::process;

use db::models::enums::{PoliticalParty, State};
use db::CreateRaceInput;

async fn seed_2020_races() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    let office = sqlx::query!(
        r#"
        SELECT id FROM office WHERE slug = 'us-senate-co-1'
    "#
    )
    .fetch_one(&pool.connection)
    .await
    .unwrap();

    // Copy one of the primaries and change meta data to reflect it is a general race
    // If winner_id exists on these races, update these politicians "upcoming_race_id" to reflect the win
    // If winner_id exists, nullify "upcoming_race_id" for losers

    let _us_senate_republican_primary = CreateRaceInput {
        slug: Some("us-senate-republican-primary-2020-colorado".to_string()),
        title: "U.S. Senate Republican Primary".to_string(),
        description: Some(
            "Republican primary election for the 2022 Colorado U.S. Senate Seat".to_string(),
        ),
        ballotpedia_link: Some(
            "https://ballotpedia.org/United_States_Senate_election_in_Colorado,_2022".to_string(),
        ),
        early_voting_begins_date: None,
        party: Some(PoliticalParty::Republican),
        office_id: office.id,
        official_website: None,
        race_type: db::models::enums::RaceType::Primary,
        state: Some(State::CO),
        winner_id: None,
        election_id: None,
    };

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = seed_2020_races().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
