use colored::*;
use db::models::enums::{PoliticalScope, State};
use db::{Chamber, District, ElectionScope};
use spinners::{Spinner, Spinners};
use std::collections::HashMap;
use std::error::Error;
use std::process;
use std::time::Instant;

async fn seed_house_and_senate_offices() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let mut sp = Spinner::new(Spinners::Dots5, "Creating offices".into());
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    let states = sqlx::query!(
        r#"
            SELECT code AS "code:State", name FROM us_states
        "#
    )
    .fetch_all(&pool.connection)
    .await?;

    let mut state_districts = HashMap::new();

    {
        state_districts.insert("Alabama", 7);
        state_districts.insert("Alaska", 1);
        state_districts.insert("Arizona", 9);
        state_districts.insert("Arkansas", 4);
        state_districts.insert("California", 52);
        state_districts.insert("Colorado", 8);
        state_districts.insert("Connecticut", 5);
        state_districts.insert("Delaware", 1);
        state_districts.insert("Florida", 28);
        state_districts.insert("Georgia", 14);
        state_districts.insert("Hawaii", 2);
        state_districts.insert("Idaho", 2);
        state_districts.insert("Illinois", 17);
        state_districts.insert("Indiana", 9);
        state_districts.insert("Iowa", 4);
        state_districts.insert("Kansas", 4);
        state_districts.insert("Kentucky", 6);
        state_districts.insert("Louisiana", 6);
        state_districts.insert("Maine", 2);
        state_districts.insert("Maryland", 8);
        state_districts.insert("Massachusetts", 9);
        state_districts.insert("Michigan", 13);
        state_districts.insert("Minnesota", 8);
        state_districts.insert("Mississippi", 4);
        state_districts.insert("Missouri", 8);
        state_districts.insert("Montana", 2);
        state_districts.insert("Nebraska", 3);
        state_districts.insert("Nevada", 4);
        state_districts.insert("New Hampshire", 2);
        state_districts.insert("New Jersey", 12);
        state_districts.insert("New Mexico", 3);
        state_districts.insert("New York", 26);
        state_districts.insert("North Carolina", 14);
        state_districts.insert("North Dakota", 1);
        state_districts.insert("Ohio", 15);
        state_districts.insert("Oklahoma", 5);
        state_districts.insert("Oregon", 6);
        state_districts.insert("Pennsylvania", 17);
        state_districts.insert("Rhode Island", 2);
        state_districts.insert("South Carolina", 7);
        state_districts.insert("South Dakota", 1);
        state_districts.insert("Tennessee", 9);
        state_districts.insert("Texas", 38);
        state_districts.insert("Utah", 4);
        state_districts.insert("Vermont", 1);
        state_districts.insert("Virginia", 11);
        state_districts.insert("Washington", 10);
        state_districts.insert("West Virginia", 2);
        state_districts.insert("Wisconsin", 8);
        state_districts.insert("Wyoming", 1);
    }

    // Loop through states and create two senate offices for each
    for state in states {
        let code = state.code;
        let state_name = state.name.unwrap();
        for i in state_districts.get(state_name.as_str()) {
            for j in 0..*i {
                let slug = format!("us-house-{}-{}", code.to_string().to_lowercase(), j + 1);
                let title = "U.S. Representative".to_string();
                let name = "U.S. House".to_string();
                let office_type = "Congressional".to_string();
                let subtitle = format!("{} - District {}", code.clone(), j + 1);
                let subtitle_short = format!("{} - {}", code.clone(), j + 1);
                let district_type = District::UsCongressional;

                sqlx::query!(
                    r#"
                INSERT INTO office (
                    slug,
                    title,
                    name,
                    office_type,
                    state,
                    political_scope,
                    election_scope,
                    chamber,
                    priority,
                    subtitle,
                    subtitle_short,
                    district_type
                
                ) VALUES
                ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                ON CONFLICT DO NOTHING
                "#,
                    slug,
                    title,
                    name,
                    office_type,
                    code as State,
                    PoliticalScope::Federal as PoliticalScope,
                    ElectionScope::District as ElectionScope,
                    Chamber::House as Chamber,
                    i + 1,
                    subtitle,
                    subtitle_short,
                    district_type as District
                )
                .execute(&pool.connection)
                .await?;
            }
        }

        for i in 0..2 {
            let slug = format!("us-senate-{}-{}", code.to_string().to_lowercase(), i + 1);
            let title = "U.S. Senator".to_string();
            let name = "U.S. Senate".to_string();
            let office_type = "Congressional".to_string();
            let subtitle = state_name.clone();
            let subtitle_short = code.to_string();

            sqlx::query!(
                r#"
            INSERT INTO office (
                slug,
                title,
                name,
                office_type,
                state,
                political_scope,
                election_scope,
                chamber,
                priority,
                subtitle,
                subtitle_short
            
            ) VALUES
            ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT DO NOTHING
            "#,
                slug,
                title,
                name,
                office_type,
                code as State,
                PoliticalScope::Federal as PoliticalScope,
                ElectionScope::State as ElectionScope,
                Chamber::Senate as Chamber,
                2,
                subtitle,
                subtitle_short
            )
            .execute(&pool.connection)
            .await?;
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
    if let Err(err) = seed_house_and_senate_offices().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
