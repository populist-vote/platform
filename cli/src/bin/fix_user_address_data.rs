use colored::*;
use db::{models::enums::State, Address};
use geocodio::GeocodioProxy;
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::process;
use std::time::Instant;

async fn fix_user_address_data() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let mut sp = Spinner::new(Spinners::Dots5, "Fixing user address data".into());
    db::init_pool().await.unwrap();
    let pool = db::pool().await;
    let addresses = sqlx::query_as!(
        Address,
        r#"
        SELECT
            a.id,
            a.line_1,
            a.line_2,
            a.city,
            a.county,
            a.state AS "state:State",
            a.postal_code,
            a.country,
            a.congressional_district,
            a.state_senate_district,
            a.state_house_district
        FROM
            address AS a
            JOIN user_profile ON address_id = a.id
        "#
    )
    .fetch_all(&pool.connection)
    .await?;

    for address in addresses {
        let address_clone = address.clone();
        let geocodio = GeocodioProxy::new().unwrap();
        let geocode_result = geocodio
            .geocode(
                geocodio::AddressParams::AddressInput(geocodio::AddressInput {
                    line_1: address_clone.line_1,
                    line_2: address_clone.line_2,
                    city: address_clone.city,
                    state: address_clone.state.to_string(),
                    country: address_clone.country,
                    postal_code: address_clone.postal_code,
                }),
                Some(&["cd118", "stateleg-next"]),
            )
            .await;

        match geocode_result {
            Ok(geocodio_data) => {
                let coordinates = geocodio_data.results[0].location.clone();
                let primary_result = geocodio_data.results[0]
                    .fields
                    .as_ref()
                    .expect(format!("{}", address_clone.id).as_str());
                let congressional_district = &primary_result
                    .congressional_districts
                    .as_ref()
                    .expect(&format!("bunk cong dist: {:?}", primary_result))[0]
                    .district_number;
                let state_legislative_districts =
                    primary_result.state_legislative_districts.as_ref();
                let state_house_district = match state_legislative_districts {
                    Some(state_legislative_districts) => {
                        Some(&state_legislative_districts.house[0].district_number)
                    }
                    None => None,
                };
                let state_senate_district = match state_legislative_districts {
                    Some(state_legislative_districts) => {
                        Some(&state_legislative_districts.senate[0].district_number)
                    }
                    None => None,
                };

                let _update_address = sqlx::query!(
                    r#"
                    UPDATE address SET
                        lat = $1,
                        lon = $2,
                        congressional_district = $3,
                        state_house_district = $4,
                        state_senate_district = $5,
                        geom = ST_GeomFromText($6, 4326)
                    WHERE id = $7
                    "#,
                    coordinates.latitude,
                    coordinates.longitude,
                    congressional_district.to_string(),
                    state_house_district,
                    state_senate_district,
                    format!("POINT({} {})", coordinates.longitude, coordinates.latitude),
                    address.id
                )
                .execute(&pool.connection)
                .await
                .unwrap();
            }
            Err(e) => {
                println!("Error for address: {}: {}", address_clone.id, e);
                process::exit(1);
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
    if let Err(err) = fix_user_address_data().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
