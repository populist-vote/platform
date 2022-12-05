use std::error::Error;
use std::process;
use votesmart::VotesmartProxy;

async fn fetch_ratings() -> Result<(), Box<dyn Error>> {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    let politicians = sqlx::query!(
        r#"
            SELECT id, votesmart_candidate_id FROM politician
        "#
    )
    .fetch_all(&pool.connection)
    .await?;

    let proxy = VotesmartProxy::new().unwrap();

    for politician in politicians {
        match politician.votesmart_candidate_id {
            Some(vs_id) => {
                let response = proxy.rating().get_candidate_rating(vs_id, None).await?;

                if response.status().is_success() {
                    let json = response
                        .json::<serde_json::Value>()
                        .await
                        .unwrap_or_default();
                    let ratings = &json["candidateRating"]["rating"];
                    let updated_politician = sqlx::query!(
                        r#"
                            UPDATE politician
                            SET votesmart_candidate_ratings = $1
                            WHERE id = $2
                            RETURNING first_name, last_name
                            "#,
                        ratings,
                        politician.id
                    )
                    .fetch_one(&pool.connection)
                    .await?;
                    println!(
                        "Ratings fetched successfully for {} {}",
                        updated_politician.first_name, updated_politician.last_name
                    );
                }
            }
            None => continue,
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = fetch_ratings().await {
        println!("Error fetching ratings: {}", err);
        process::exit(1);
    }
}
