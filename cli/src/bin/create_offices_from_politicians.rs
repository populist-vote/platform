use db::models::enums::State;
use db::{models::enums::PoliticalScope, CreateOfficeInput, Office};
use serde::{Deserialize, Serialize};
use slugify::slugify;
use std::error::Error;
use std::process;
use std::str::FromStr;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct VotesmartOffice {
    name: Vec<String>,
    #[serde(rename = "type")]
    type_field: String,
    title: String,
    status: String,
    parties: String,
    #[serde(rename = "stateId")]
    state_id: String,
    district: String,
}

async fn create_offices() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    let politicians = sqlx::query!(
        r#"
            SELECT id, votesmart_candidate_bio->'office' AS office FROM politician
        "#
    )
    .fetch_all(&pool.connection)
    .await?;

    for politician in politicians {
        let office: VotesmartOffice = serde_json::from_value(politician.office.unwrap()).unwrap();
        // println!("{}", serde_json::to_string_pretty(&office).unwrap());

        let political_scope = match office.type_field.as_ref() {
            "Local Executive" => PoliticalScope::Local,
            "Gubernatorial" => PoliticalScope::State,
            "State Legislative" => PoliticalScope::State,
            "Congressional" => PoliticalScope::Federal,
            _ => PoliticalScope::Federal,
        };

        let new_office_input = CreateOfficeInput {
            slug: Some(slugify!(
                format!("{} {}", office.name[0], office.district).as_str()
            )),
            title: office.name.first().unwrap().to_owned(),
            office_type: Some(office.type_field),
            district: Some(office.district),
            political_scope,
            state: Some(State::from_str(&office.state_id).unwrap()),
            encumbent_id: politician.id,
        };

        Office::create(&pool.connection, &new_office_input).await?;
        println!("Office record has been created: {}", office.title);
    }
    println!("Offices have been successfully seeded.");
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = create_offices().await {
        println!("Error seeding offices: {}", err);
        process::exit(1);
    }
}
