use colored::*;
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::process;
use std::time::Instant;

async fn create_fec_senate_candidates() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;
    let start = Instant::now();
    let mut sp = Spinner::new(
        Spinners::Dots5,
        "Creating politician records and race_candidate records \n".into(),
    );

    // sqlx::query!(
    //     r#"
    //     INSERT INTO politician (first_name, last_name, suffix, slug, home_state, fec_candidate_id, party_id)
    //     SELECT DISTINCT ON (candidate_id)
    //         first_name,
    //         last_name,
    //         suffix,
    //         generate_unique_slug(slugify(CONCAT(first_name, ' ', last_name, ' ', suffix, ' ', state)), 'politician'),
    //         state::state,
    //         candidate_id,
    //         party_id
    //     FROM
    //         dbt_models.stg_fec_federal_politicians
    //     WHERE politician_id IS NULL;
    // "#
    // )
    // .execute(&pool.connection)
    // .await?;

    // must run dbt in between these queries

    // sqlx::query!(
    //     r#"
    //         INSERT INTO race_candidates (race_id, candidate_id)
    //         SELECT race_id, politician_id FROM dbt_models.stg_fec_federal_politicians;
    //     "#
    // )
    // .execute(&pool.connection)
    // .await?;

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
    if let Err(err) = create_fec_senate_candidates().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
