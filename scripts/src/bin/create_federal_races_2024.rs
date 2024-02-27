use chrono::NaiveDate;
use colored::*;
use db::models::enums::{FullState, PoliticalParty, State};
use spinners::{Spinner, Spinners};
use std::collections::HashMap;
use std::error::Error;
use std::str::FromStr;
use std::time::Instant;
use std::{env, process};

async fn create_federal_house_primaries_2024() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let mut sp = Spinner::new(Spinners::Dots5, "Creating races".into());
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    let api_key = env::var("OPEN_FEC_API_KEY").unwrap();
    let election_year = 2024;
    let election_type_id = "P";
    let office_sought = "H";
    let url = format!("https://api.open.fec.gov/v1/election-dates?api_key={}&election_year={}&election_type_id={}&office_sought={}&sort=election_date&per_page=100",
        api_key, election_year, election_type_id, office_sought);

    let json = reqwest::get(&url)
        .await?
        .json::<serde_json::Value>()
        .await?;

    let results = json["results"].as_array().unwrap();

    // Creates elections for each state
    for result in results {
        let state = result["election_state"].as_str();

        if state.is_none() {
            continue;
        }
        let state = State::from_str(state.unwrap());

        if let Ok(state) = state {
            let state_full = state.full_state();
            let title = format!("{} Primaries 2024", state_full);
            let slug = format!("{}-primaries-2024", state_full.to_lowercase());
            let description = format!(
                "Primary races in {} for the upcoming general election on November 5, 2024",
                state_full
            );
            let date = result["election_date"].as_str().unwrap();
            sqlx::query!(
                r#"
                INSERT INTO election (slug, title, description, election_date, state)
                VALUES (slugify($1), $2, $3, $4, $5::state)
                ON CONFLICT (slug) DO NOTHING
            "#,
                slug,
                title,
                description,
                NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
                state as State
            )
            .execute(&pool.connection)
            .await?;
        } else {
            tracing::error!("Could not parse state: {}", state.unwrap_err());
        }
    }

    let mut election_types = HashMap::new();
    {
        election_types.insert("AL", "partisan");
        election_types.insert("AK", "top-four");
        election_types.insert("AZ", "partisan");
        election_types.insert("AR", "partisan");
        election_types.insert("CA", "top-two");
        election_types.insert("CO", "partisan");
        election_types.insert("CT", "partisan");
        election_types.insert("DE", "partisan");
        election_types.insert("FL", "partisan");
        election_types.insert("GA", "partisan");
        election_types.insert("HI", "partisan");
        election_types.insert("ID", "partisan");
        election_types.insert("IL", "partisan");
        election_types.insert("IN", "partisan");
        election_types.insert("IA", "partisan");
        election_types.insert("KS", "partisan");
        election_types.insert("KY", "partisan");
        election_types.insert("LA", "N/A");
        election_types.insert("ME", "partisan");
        election_types.insert("MD", "partisan");
        election_types.insert("MA", "partisan");
        election_types.insert("MI", "partisan");
        election_types.insert("MN", "partisan");
        election_types.insert("MS", "partisan");
        election_types.insert("MO", "partisan");
        election_types.insert("MT", "partisan");
        election_types.insert("NE", "top-two");
        election_types.insert("NV", "partisan");
        election_types.insert("NH", "partisan");
        election_types.insert("NJ", "partisan");
        election_types.insert("NM", "partisan");
        election_types.insert("NY", "partisan");
        election_types.insert("NC", "partisan");
        election_types.insert("ND", "partisan");
        election_types.insert("OH", "partisan");
        election_types.insert("OK", "partisan");
        election_types.insert("OR", "partisan");
        election_types.insert("PA", "partisan");
        election_types.insert("RI", "partisan");
        election_types.insert("SC", "partisan");
        election_types.insert("SD", "partisan");
        election_types.insert("TN", "partisan");
        election_types.insert("TX", "partisan");
        election_types.insert("UT", "partisan");
        election_types.insert("VT", "partisan");
        election_types.insert("VA", "partisan");
        election_types.insert("WA", "top-two");
        election_types.insert("WV", "partisan");
        election_types.insert("WI", "partisan");
        election_types.insert("WY", "partisan");
    }

    for (state_code, election_type) in election_types.iter() {
        // Get office_id of house offices for given state
        let state_code = State::from_str(&state_code).unwrap();
        let offices = sqlx::query!(
            r#"
            SELECT id, name, district FROM office WHERE state = $1::state AND title = 'U.S. Representative'
        "#,
            state_code as State
        )
        .fetch_all(&pool.connection)
        .await?;

        for office in offices {
            let office_id = office.id;

            match election_type {
                &"partisan" => {
                    // Create a republican and democratic primary race for current house office
                    for party in vec![PoliticalParty::Republican, PoliticalParty::Democratic] {
                        // U.S. House - CO - 4 - Democratic Primary

                        let title = format!(
                            "U.S. House - {} - {} - Primary - {} - 2024",
                            state_code,
                            office
                                .district
                                .clone()
                                .expect(format!("No district for office: {:?}", office).as_str()),
                            party
                        );

                        sqlx::query!(
                            r#"
                            INSERT INTO race (slug, title, description, election_id, office_id, party, race_type)
                            VALUES (slugify(REPLACE($1, '.', '')), $2, $3, (SELECT id FROM election WHERE slug = slugify($4)), $5, $6::political_party, 'primary')
                        "#,
                            title,
                            title,
                            title,
                            format!("{}-primaries-2024", state_code.full_state().to_lowercase()),
                            office_id,
                            party as PoliticalParty
                        ).execute(&pool.connection).await?;
                    }
                }
                &"top-four" => {
                    let title = format!(
                        "U.S. House - {} - {} - Primary - 2024",
                        state_code,
                        office
                            .district
                            .clone()
                            .expect(format!("No district for office: {:?}", office).as_str()),
                    );
                    sqlx::query!(
                        r#"
                        INSERT INTO race (slug, title, description, election_id, office_id, race_type, num_elect)
                        VALUES (slugify(REPLACE($1, '.', '')), $2, $3, (SELECT id FROM election WHERE slug = slugify($4)), $5, 'primary', 4)
                        "#,
                        title,
                        title,
                        title,
                        format!("{}-primaries-2024", state_code.full_state().to_lowercase()),
                        office_id,
                    ).execute(&pool.connection).await?;
                }
                &"top-two" => {
                    let title = format!(
                        "U.S. House - {} - {} - Primary - 2024",
                        state_code,
                        office.district.clone().unwrap(),
                    );
                    sqlx::query!(
                        r#"
                        INSERT INTO race (slug, title, description, election_id, office_id, race_type, num_elect)
                        VALUES (slugify(REPLACE($1, '.', '')), $2, $3, (SELECT id FROM election WHERE slug = slugify($4)), $5, 'primary', 2)
                        "#,
                        title,
                        title,
                        title,
                        format!("{}-primaries-2024", state_code.full_state().to_lowercase()),
                        office_id,
                    ).execute(&pool.connection).await?;
                }
                _ => {
                    // No primary
                }
            }
        }
    }
    sp.stop();
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

#[tokio::main]
async fn main() {
    if let Err(err) = create_federal_house_primaries_2024().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
