use colored::*;
use open_fec::{CandidatesQuery, OpenFecProxy};
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::process;
use std::time::Instant;

async fn run() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let mut sp = Spinner::new(Spinners::Dots5, "Getting candidates".into());
    let proxy = OpenFecProxy::new().unwrap();
    let query = CandidatesQuery {
        office: Some("S".to_string()),
        state: Some("NY".to_string()),
        ..CandidatesQuery::default()
    };
    let res = proxy.get_candidates(query).await?;
    if !res.status().is_success() {
        println!("Error: {}", res.status());
    }

    let json: serde_json::Value = res.json().await?;
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
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
    if let Err(err) = run().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
