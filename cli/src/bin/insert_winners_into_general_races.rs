use std::error::Error;
use std::process;

async fn insert_primary_winners_into_general_races() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    let office_primary_winners = sqlx::query!(
        r#"
        SELECT
            office_id,
            winner_id
        FROM
            race r
            JOIN office ON r.office_id = office.id
        WHERE
            winner_id IS NOT NULL
            AND race_type = 'primary'
            AND r.office_id = office.id
   "#
    )
    .fetch_all(&pool.connection)
    .await?;

    for winner in office_primary_winners {
        sqlx::query!(
            r#"
            INSERT INTO race_candidates (race_id, candidate_id)
            VALUES((SELECT id FROM race WHERE race.office_id = $1 AND race.race_type = 'general' LIMIT 1), $2)
            ON CONFLICT DO NOTHING
        "#,
            winner.office_id,
            winner.winner_id
        )
        .fetch_optional(&pool.connection)
        .await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = insert_primary_winners_into_general_races().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
