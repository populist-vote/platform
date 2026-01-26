use scrapers::processors::mn::candidate_filings::process_mn_candidate_filings;

#[tokio::main]
async fn main() {
    // Initialize database connection
    db::init_pool().await.unwrap();
    let pool = db::pool().await;
    
    println!("=== MN Candidate Filings Processor ===\n");
    
    // Process the filings
    match process_mn_candidate_filings(
        &pool.connection,
        "mn_candidate_filings_local_2025",
        "general"
    ).await {
        Ok(_) => {
            println!("\n✓ Processing completed successfully!");
            println!("\nYou can now examine the staging tables:");
            println!("  SELECT * FROM dbt_henry.stg_offices;");
            println!("  SELECT * FROM dbt_henry.stg_politicians;");
            println!("  SELECT * FROM dbt_henry.stg_races;");
            println!("  SELECT * FROM dbt_henry.stg_race_candidates;");
        },
        Err(e) => {
            eprintln!("\n✗ Error processing filings: {}", e);
            std::process::exit(1);
        }
    }
}

