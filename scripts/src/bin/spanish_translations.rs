use colored::*;
use db::util::translate::translate_text;
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::process;
use std::time::Instant;

async fn spanish_translations() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let mut sp = Spinner::new(Spinners::Dots5, "Translating responses into spanish".into());

    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    let responses = sqlx::query!(
        r#"
            WITH guide_averages AS (
            SELECT
                qs.id AS submission_id,
                r.title AS race,
                p.id AS politician_id,
                p.first_name,
                p.last_name,
                qs.response AS response,
                qs.editorial AS editorial,
                qs.translations AS translations
            FROM
                candidate_guide cg
                JOIN candidate_guide_questions cgq ON cg.id = cgq.candidate_guide_id
                JOIN question q ON cgq.question_id = q.id
                LEFT JOIN question_submission qs ON q.id = qs.question_id
                JOIN politician p ON qs.candidate_id = p.id
                JOIN race_candidates rc ON p.id = rc.candidate_id
                JOIN race r ON rc.race_id = r.id
            WHERE
                cg.organization_id = (
                    SELECT
                        id
                    FROM
                        organization
                    WHERE
                        slug = 'mpr-news')
                AND qs.translations->>'temp' IS NULL
        )
        SELECT
            submission_id, response, editorial
        FROM
            guide_averages
        ORDER BY race, last_name
        "#
    )
    .fetch_all(&pool.connection)
    .await?;

    for response in responses {
        if response.response.is_empty() {
            continue;
        }
        let result = translate_text(&response.response, vec!["es"]).await;
        if let Ok(result) = result {
            sqlx::query!(
                r#"
                    UPDATE question_submission
                    SET translations = jsonb_set(
                        COALESCE(translations, '{}'),  -- Default to empty JSON object if NULL
                        '{temp}', 
                        $1::jsonb, 
                        true
                    )
                    WHERE id = $2;
                "#,
                result,
                response.submission_id
            )
            .execute(&pool.connection)
            .await?;
        }
    }

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
    if let Err(err) = spanish_translations().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
