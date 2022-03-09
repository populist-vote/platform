use db::models::enums::State;
use db::{models::enums::PoliticalScope, CreateOfficeInput, Office};
use db::{Politician, UpdatePoliticianInput};
use rand::Rng;
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
    let mut rng = rand::thread_rng();
    let pool = db::pool().await;

    let politicians = sqlx::query!(
        r#"
            SELECT id, votesmart_candidate_bio->'office' AS office FROM politician
        "#
    )
    .fetch_all(&pool.connection)
    .await?;

    for politician in politicians {
        if politician.office.is_none() {
            continue;
        };

        let office: VotesmartOffice =
            serde_json::from_value(politician.office.unwrap_or_default()).unwrap_or_default();
        // println!("{}", serde_json::to_string_pretty(&office).unwrap());

        let political_scope = match office.type_field.as_ref() {
            "Local Executive" => PoliticalScope::Local,
            "Gubernatorial" => PoliticalScope::State,
            "State Legislative" => PoliticalScope::State,
            "Congressional" => PoliticalScope::Federal,
            _ => PoliticalScope::Federal,
        };

        let new_office_input = CreateOfficeInput {
            slug: Some(slugify!(format!(
                "{} {} {}",
                office
                    .name
                    .first()
                    .unwrap_or(&"unknown".to_string())
                    .to_owned(),
                office.district,
                rng.gen::<i16>()
            )
            .replace(".", "")
            .as_str())),
            title: office
                .name
                .first()
                .unwrap_or(&format!("unknown-{}", rng.gen::<u32>()).to_string())
                .to_owned(),
            office_type: Some(office.type_field),
            district: Some(office.district),
            municipality: None,
            term_length: None,
            political_scope,
            state: Some(State::from_str(&office.state_id).unwrap_or_default()),
            incumbent_id: Some(politician.id),
        };

        let new_office = Office::create(&pool.connection, &new_office_input).await?;
        let update_politician_input = UpdatePoliticianInput {
            office_id: Some(new_office.id),
            ..Default::default()
        };

        Politician::update(
            &pool.connection,
            Some(politician.id),
            None,
            &update_politician_input,
        )
        .await?;
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
