use db::models::enums::State;
use db::{models::enums::PoliticalScope, CreateOfficeInput, Office};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::process;
use std::str::FromStr;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
struct VotesmartOffice {
    name: Vec<String>,
    #[serde(rename = "type")]
    type_field: String,
    state: String,
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
        // Use these fields to create new Offices
        // Use office_type
        let political_scope = match office.type_field.as_ref() {
            "Mayor" => PoliticalScope::Local,
            _ => PoliticalScope::Federal,
        };

        let new_office_input = CreateOfficeInput {
            slug: None,
            title: office.name.first().unwrap().to_owned(),
            office_type: Some(office.type_field),
            political_scope,
            state: Some(State::from_str(&office.state).unwrap()),
            encumbent_id: politician.id,
        };
        Office::create(&pool.connection, &new_office_input).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = create_offices().await {
        println!("Error seeding bills: {}", err);
        process::exit(1);
    }
}
