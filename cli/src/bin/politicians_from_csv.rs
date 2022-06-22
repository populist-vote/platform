use chrono::NaiveDate;
use db::models::enums::PoliticalParty;
use db::models::enums::State;
use db::CreatePoliticianInput;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::io;
use std::process;
use votesmart::VotesmartProxy;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PoliticianRow {
    pub slug: String,
    pub first_name: String,
    pub middle_name: Option<String>,
    pub last_name: String,
    pub suffix: Option<String>,
    pub preferred_name: Option<String>,
    pub biography: Option<String>,
    pub biography_source: Option<String>,
    pub home_state: Option<State>,
    pub date_of_birth: Option<String>,
    pub office_id: Option<uuid::Uuid>,
    pub thumbnail_image_url: Option<String>,
    pub website_url: Option<String>,
    pub campaign_website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub youtube_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub tiktok_url: Option<String>,
    pub email: Option<String>,
    pub party: Option<PoliticalParty>,
    pub votesmart_candidate_id: Option<i32>,
    pub votesmart_candidate_bio: Option<Value>,
    pub votesmart_candidate_ratings: Option<Value>,
    pub legiscan_people_id: Option<i32>,
    pub crp_candidate_id: Option<String>,
    pub fec_candidate_id: Option<String>,
    pub race_wins: Option<i32>,
    pub race_losses: Option<i32>,
    pub upcoming_race_id: Option<uuid::Uuid>,
}

async fn example() -> Result<(), Box<dyn Error>> {
    // Init database connection singleton
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    // Build the CSV reader and iterate over each record.
    let mut rdr = csv::Reader::from_reader(io::stdin());
    for result in rdr.deserialize() {
        let mut new_record_input: PoliticianRow = result?;

        let record = sqlx::query!(
            r#"
            INSERT INTO politician (slug, first_name, middle_name, last_name, suffix, preferred_name, biography, biography_source, home_state, date_of_birth, thumbnail_image_url, website_url, campaign_website_url, facebook_url, twitter_url, instagram_url, youtube_url, linkedin_url, tiktok_url, email, party, votesmart_candidate_id, legiscan_people_id, crp_candidate_id, fec_candidate_id, race_wins, race_losses, upcoming_race_id)
            VALUES($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28) 
            ON CONFLICT (slug) DO UPDATE
            SET
                suffix = $5,
                preferred_name = $6,
                biography = $7,
                biography_source = $8,
                home_state = $9,
                date_of_birth = $10,
                thumbnail_image_url = $11,
                website_url = $12,
                campaign_website_url = $13,
                facebook_url = $14,
                twitter_url = $15,
                instagram_url = $16,
                youtube_url = $17,
                linkedin_url = $18,
                tiktok_url = $19,
                email = $20,
                party = $21,
                race_wins = $26,
                race_losses = $27,
                upcoming_race_id = $28
            RETURNING id
            "#, 
            new_record_input.slug,
            new_record_input.first_name,
            new_record_input.middle_name,
            new_record_input.last_name,
            new_record_input.suffix,
            new_record_input.preferred_name,
            new_record_input.biography,
            new_record_input.biography_source,
            new_record_input.home_state as Option<State>,
            new_record_input.date_of_birth.map(|d| NaiveDate::parse_from_str(&d, "%m/%d/%Y").unwrap()),
            new_record_input.thumbnail_image_url,
            new_record_input.website_url,
            new_record_input.campaign_website_url,
            new_record_input.facebook_url,
            new_record_input.twitter_url,
            new_record_input.instagram_url,
            new_record_input.youtube_url,
            new_record_input.linkedin_url,
            new_record_input.tiktok_url,
            new_record_input.email,
            new_record_input.party as Option<PoliticalParty>,
            new_record_input.votesmart_candidate_id,
            new_record_input.legiscan_people_id,
            new_record_input.crp_candidate_id,
            new_record_input.fec_candidate_id,
            new_record_input.race_wins,
            new_record_input.race_losses,
            new_record_input.upcoming_race_id
        )
        .fetch_one(&pool.connection)
        .await;

        println!("Created or updated {}", new_record_input.last_name)
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = example().await {
        println!("error running example: {}", err);
        process::exit(1);
    }
}
