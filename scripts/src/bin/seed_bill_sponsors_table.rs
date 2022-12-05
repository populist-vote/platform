use std::error::Error;
use std::process;

async fn seed_bill_sponsors_table() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    let politician_records = sqlx::query!("SELECT id, votesmart_candidate_id FROM politician")
        .fetch_all(&pool.connection)
        .await?;

    for politician in politician_records {
        let bill_ids: Vec<uuid::Uuid> = sqlx::query!(
            r#"SELECT id FROM bill, jsonb_array_elements(legiscan_data->'sponsors') sponsors 
            WHERE sponsors->>'votesmart_id' = $1"#,
            &politician.votesmart_candidate_id.unwrap().to_string()
        )
        .fetch_all(&pool.connection)
        .await?
        .into_iter()
        .map(|r| r.id)
        .collect();

        for bill_id in bill_ids {
            sqlx::query!(
                r#"INSERT INTO bill_sponsors (bill_id, politician_id) VALUES ($1, $2)"#,
                bill_id,
                politician.id
            )
            .execute(&pool.connection)
            .await?;
        }
    }

    println!("SUCCESS: seeded bill_sponsors table");
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = seed_bill_sponsors_table().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
