use colored::*;
use open_fec::candidate::CandidateQuery;
use open_fec::candidates::CandidatesQuery;
use open_fec::OpenFecProxy;
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::process;
use std::time::Instant;

async fn run() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    println!("\n");
    let mut sp = Spinner::new(Spinners::Dots5, "Getting candidates".into());
    let proxy = OpenFecProxy::new().unwrap();
    // Decide what to query here for federal candidates, we will need to paginate through if we fetch them oll
    let query = CandidatesQuery {
        per_page: Some(100),
        is_active_candidate: Some(true),
        office: Some("P".into()),
        election_year: Some(2024),
        ..CandidatesQuery::default()
    };

    // Fetch first page of candidates

    // let res = proxy.candidates().get_candidates(Some(query)).await?;

    let res = proxy
        .candidate()
        .get_candidate("P40014052", CandidateQuery::default())
        .await?;
    if !res.status().is_success() {
        println!("Error: {}", res.status());
    } else {
        sp.stop();
    }

    // Paginate through all candidates and store them in memory

    // Create a temp table to hold all records in postgres

    // Search for existing candidates by fec_id and update if exists, insert if not
    // May be better to do this step with DBT and a materialized view

    let json: serde_json::Value = res.json().await?;

    let results = json["results"].as_array().unwrap();

    println!("{}", serde_json::to_string_pretty(&results).unwrap());

    let count = json["pagination"]["count"].as_u64().unwrap();
    // println!("{}", serde_json::to_string_pretty(&json).unwrap());
    println!("\ncount = {:?}", count);

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
    if let Err(err) = run().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
