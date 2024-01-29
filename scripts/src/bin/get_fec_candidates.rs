use std::error::Error;
use std::process;

use open_fec::candidate::CandidatesQuery;
use open_fec::OpenFecProxy;

async fn run() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    let proxy = OpenFecProxy::new().unwrap();
    // Decide what to query here for federal candidates, we will need to paginate through if we fetch them oll
    let query = CandidatesQuery {
        election_year: Some(2024),
        is_active_candidate: Some(true),
        office_sought: Some("H".into()),
        per_page: Some(100), // Max is 100
        sort: Some("candidate_status".into()),
        ..CandidatesQuery::default()
    };

    let res = proxy.get_candidates(query).await?;
    let json: serde_json::Value = res.json().await?;
    let pagination = json["pagination"].as_object().unwrap();
    let pages = pagination["pages"].as_u64().unwrap();
    // Use Vec to store candidate results
    let mut results: Vec<serde_json::Value> = Vec::new();
    // Add page 1 results to the vector
    results.extend(json["results"].as_array().unwrap().to_owned());

    for page in 2..pages + 1 {
        println!("Fetching page {} of {}", page, pages);
        let query = CandidatesQuery {
            election_year: Some(2024),
            is_active_candidate: Some(true),
            office_sought: Some("H".into()),
            per_page: Some(100), // Max is 100
            sort: Some("candidate_status".into()),
            page: Some(page.into()),
            ..CandidatesQuery::default()
        };
        let res = proxy.get_candidates(query).await?;
        let json: serde_json::Value = res.json().await.expect("Failed to parse json");
        let page_results = json["results"].as_array().unwrap();
        // Add page results to the vector
        results.extend(page_results.to_owned());
    }

    // Get column names from first result
    let column_names = results[0].as_object().unwrap().keys();

    // Create temp table in p6t_federal schema to store fec_house_candidates_2024
    let query = format!(
        r#"
        CREATE TABLE IF NOT EXISTS p6t_federal.fec_house_candidates_2024 (
            {}
        )
        "#,
        column_names
            .map(|name| format!("{} text", name))
            .collect::<Vec<String>>()
            .join(", ")
    );

    sqlx::query(&query).execute(&pool.connection).await?;

    // Insert results into temp table
    sqlx::query!(
        r#"
        INSERT INTO p6t_federal.fec_house_candidates_2024
        SELECT * FROM json_populate_recordset(null::p6t_federal.fec_house_candidates_2024, $1::json)
        "#,
        // Parse vec of json objects into string, then parse string into jsonb...ick
        serde_json::to_string(&results)?.parse::<serde_json::Value>()?
    )
    .execute(&pool.connection)
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
