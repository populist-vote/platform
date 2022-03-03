use db::{CreateOrganizationInput, Organization};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::process;
use votesmart::VotesmartProxy;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VotesmartSig {
    pub address: String,
    pub city: String,
    pub contact_name: String,
    pub description: String,
    pub email: String,
    pub fax: String,
    pub general_info: GeneralInfo,
    pub name: String,
    pub parent_id: String,
    pub phone1: String,
    pub phone2: String,
    pub sig_id: String,
    pub state: String,
    pub state_id: String,
    pub url: String,
    pub zip: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralInfo {
    pub link_back: String,
    pub title: String,
}

async fn create_organizations() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    let proxy = VotesmartProxy::new().unwrap();
    let pool = db::pool().await;

    let ratings_records = sqlx::query!(
        r#"
            SELECT votesmart_candidate_ratings FROM politician
        "#
    )
    .fetch_all(&pool.connection)
    .await?;

    // Lets get unique sig_ids so we don't blast the Votesmart API too heavy
    let unique_sig_ids = ratings_records
        .iter()
        .flat_map(|record| {
            let ratings: Vec<serde_json::Value> =
                serde_json::from_value(record.votesmart_candidate_ratings.to_owned())
                    .unwrap_or_default();
            ratings
                .iter()
                .map(|rating| rating["sigId"].as_str().unwrap().parse::<i32>().unwrap())
                .collect::<Vec<i32>>()
                .into_iter()
                .collect::<std::collections::HashSet<i32>>()
        })
        .collect::<std::collections::HashSet<i32>>();

    for sig_id in unique_sig_ids {
        // let existing_org = sqlx::query_as!(
        //     Organization,
        //     r#"
        //         SELECT * FROM organization
        //         WHERE votesmart_sig_id = $1"#,
        //     sig_id
        // )
        // .fetch_optional(&pool.connection)
        // .await
        // .unwrap();

        let vs_response = proxy.rating().get_sig(sig_id).await?;
        let json = vs_response.json::<serde_json::Value>().await.unwrap()["sig"].to_owned();
        let sig: VotesmartSig = serde_json::from_value(json).unwrap_or_default();
        let address_line_1 = sig.address.split(",").collect::<Vec<&str>>()[0].to_string();
        let temp_address = sig.address.split(",").collect::<Vec<&str>>();
        let address_line_2 = temp_address.get(1).unwrap_or(&"").to_string();
        let city = sig.city;
        let state = sig.state;
        let zip = sig.zip;
        let description = sig.description;
        let email = sig.email;
        let name = sig.name;
        let website = sig.url;
        let phone = sig.phone1;

        let existing_org = sqlx::query_as!(
            Organization,
            r#"
                UPDATE organization SET votesmart_sig_id = $1 
                WHERE name ILIKE $2
                "#,
            sig_id,
            name
        )
        .fetch_optional(&pool.connection)
        .await;
        // .unwrap();

        // if let Some(org) = existing_org {
        //     sqlx::query!(
        //         r#"
        //             WITH new_address_id AS (
        //                 INSERT INTO address (
        //                     line_1,
        //                     line_2,
        //                     city,
        //                     state,
        //                     postal_code,
        //                     country
        //                 ) VALUES (
        //                     $1,
        //                     $2,
        //                     $3,
        //                     $4,
        //                     $5,
        //                     $6
        //                 ) RETURNING id
        //             ),
        //             org AS (
        //                 UPDATE organization
        //                 SET headquarters_address_id = (SELECT new_address_id.id FROM new_address_id)
        //                 WHERE id = $7
        //                 RETURNING id
        //             )
        //             SELECT org.* FROM org
        //         "#,
        //         address_line_1,
        //         address_line_2,
        //         city,
        //         state,
        //         zip,
        //         "USA",
        //         org.id
        //     )
        //     .fetch_one(&pool.connection)
        //     .await?;

        //     println!("Updated {}", name);
        // } else {
        //     let new_address = sqlx::query!(
        //         r#"
        //             INSERT INTO address (
        //                 line_1,
        //                 line_2,
        //                 city,
        //                 state,
        //                 postal_code,
        //                 country
        //             ) VALUES (
        //                 $1,
        //                 $2,
        //                 $3,
        //                 $4,
        //                 $5,
        //                 $6
        //             ) RETURNING id
        //         "#,
        //         address_line_1,
        //         address_line_2,
        //         city,
        //         state,
        //         zip,
        //         "USA"
        //     )
        //     .fetch_one(&pool.connection)
        //     .await?;

        //     let new_org_input = CreateOrganizationInput {
        //         name: name.clone(),
        //         slug: None,
        //         thumbnail_image_url: None,
        //         facebook_url: None,
        //         twitter_url: None,
        //         instagram_url: None,
        //         headquarters_phone: Some(phone),
        //         tax_classification: None,
        //         issue_tags: None,
        //         website_url: Some(website),
        //         description: Some(description),
        //         email: Some(email),
        //         votesmart_sig_id: Some(sig_id),
        //         headquarters_address_id: Some(new_address.id),
        //     };
        //     if let Err(err) = Organization::create(&pool.connection, &new_org_input).await {
        //         println!("Error creating {}: {}", name, err);
        //     } else {
        //         println!("Created {}", name);
        //     }
        // }
    }

    println!("Organizations have been successfully seeded.");
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = create_organizations().await {
        println!("Error seeding offices: {}", err);
        process::exit(1);
    }
}
