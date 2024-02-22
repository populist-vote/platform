use std::error::Error;
use std::process;

async fn seed_bill_sponsors_table() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    let query = sqlx::query!(
        r#"
            WITH politician_records AS (
                SELECT p.id, p.legiscan_people_id, b.id AS bill_id
                FROM politician AS p
                LEFT JOIN bill AS b ON b.legiscan_data->'sponsors' @> jsonb_build_array(jsonb_build_object('people_id', p.legiscan_people_id))
                WHERE p.legiscan_people_id IS NOT NULL AND b.id IS NOT NULL
            )
            INSERT INTO bill_sponsors (bill_id, politician_id)
            SELECT bill_id, id
            FROM politician_records
            ON CONFLICT DO NOTHING
        "#
    );

    query.execute(&pool.connection).await?;

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
