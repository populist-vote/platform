use colored::*;
use serde::{Deserialize, Serialize};
use spinners::{Spinner, Spinners};
use std::collections::HashMap;
use std::error::Error;
use std::io;
use std::process;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ColoradoSummaryResult {
    #[serde(rename = "choice name")]
    full_name: String,
    #[serde(rename = "total votes")]
    total_votes: i32,
    #[serde(rename = "id (from Politicians)")]
    politician_id: uuid::Uuid,
}

async fn import_race_results_from_csv() -> Result<(), Box<dyn Error>> {
    let mut sp = Spinner::new(Spinners::Dots5, "Importing race results from CSV".into());
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    let mut total_counts = HashMap::new();

    let mut rdr = csv::Reader::from_reader(io::stdin());
    for result in rdr.deserialize() {
        let candidate: ColoradoSummaryResult = result?;
        let record = sqlx::query!(
            r#"
            WITH rc AS (
                SELECT candidate_id, race_id FROM race_candidates
                JOIN politician ON politician.id = candidate_id
                JOIN race ON race.id = race_id
                JOIN election ON election.id = election_id
                WHERE election.slug = 'general-election-2022' AND
                      candidate_id = $2
            )
            UPDATE race_candidates
            SET votes = $1
            FROM rc
            WHERE race_candidates.candidate_id = rc.candidate_id AND
                  race_candidates.race_id = rc.race_id
            RETURNING race_candidates.race_id
        "#,
            candidate.total_votes,
            candidate.politician_id
        )
        .fetch_optional(&pool.connection)
        .await?;

        if record.is_none() {
            eprintln!(
                "⚠️  {}",
                format!("\nNO RECORD FOUND FOR {}", candidate.full_name)
                    .bright_yellow()
                    .bold()
            )
        }

        if let Some(record) = record {
            if let Some(count) = total_counts.get_mut(&record.race_id) {
                *count += candidate.total_votes;
            } else {
                total_counts.insert(record.race_id.clone(), candidate.total_votes);
            }
        }
    }

    // for (race_id, count) in &total_counts {
    //     let _query = sqlx::query!(
    //         r#"
    //         UPDATE race
    //         SET total_votes = $1,
    //             winner_ids = (SELECT candidate_id FROM race_candidates WHERE race_id = $2 ORDER BY votes DESC NULLS LAST LIMIT 1)
    //         WHERE id = $2
    //         "#,
    //         count,
    //         race_id
    //     )
    //     .fetch_optional(&pool.connection)
    //     .await;
    // }

    // Update race wins
    // let _query = sqlx::query!(
    //     r#"
    //         UPDATE
    //             politician AS p
    //         SET
    //             race_wins = race_wins + 1
    //         FROM
    //             race AS r
    //         WHERE
    //             r.winner_id = p.id
    //             AND((
    //                 SELECT
    //                     COUNT(*)
    //                     FROM race_candidates rc
    //                 WHERE
    //                     rc.race_id = r.id) > 1);
    //     "#
    // )
    // .fetch_optional(&pool.connection)
    // .await;

    // // Update race losses
    // let _query = sqlx::query!(
    //     r#"
    //         UPDATE
    //             politician AS p
    //         SET
    //             race_losses = race_losses + 1
    //         FROM
    //             race_candidates AS rc
    //         WHERE
    //             rc.candidate_id = p.id
    //             AND(
    //                 SELECT
    //                     winner_id FROM race
    //                 WHERE
    //                     race.id = rc.race_id) IS NOT NULL
    //             AND(
    //                 SELECT
    //                     winner_id FROM race
    //                 WHERE
    //                     race.id = rc.race_id) != p.id;
    //     "#
    // )
    // .fetch_optional(&pool.connection)
    // .await;

    sp.stop();
    eprintln!("\n✅ {}", "Success".bright_green().bold());
    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(err) = import_race_results_from_csv().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
