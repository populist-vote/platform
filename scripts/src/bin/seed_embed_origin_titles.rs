use colored::*;
use regex::Regex;
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::process;
use std::time::Instant;

async fn seed_embed_origin_titles() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let mut sp = Spinner::new(Spinners::Dots5, "Fetching titles".into());
    db::init_pool().await.unwrap();
    let db_pool = db::pool().await;

    let origins = sqlx::query!(
        r#"
        SELECT url
        FROM embed_origin
        WHERE page_title IS NULL
        "#
    )
    .fetch_all(&db_pool.connection)
    .await?;

    for origin in origins {
        let url = origin.url;

        let html = reqwest::get(url.clone()).await;

        match html {
            Ok(html) => {
                let html = html.text().await?;
                // Regex to capture the content within the <title> tag
                let re = Regex::new(r"(?i)<title>(.*?)</title>").unwrap(); // (?i) makes it case-insensitive

                // Extract the title using the regex
                let fetched_page_title = re
                    .captures(&html)
                    .and_then(|caps| caps.get(1).map(|title| title.as_str().trim().to_string()))
                    .unwrap_or_default(); // Return an empty string if no title is found

                // Update the record in the database with the fetched page title
                sqlx::query!(
                    r#"
            UPDATE embed_origin
            SET page_title = $1
            WHERE url = $2
            "#,
                    fetched_page_title,
                    url
                )
                .execute(&db_pool.connection)
                .await?;
            }
            Err(e) => {
                eprintln!("Failed to fetch or update the title for {}: {}", url, e);
                continue;
            }
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
    if let Err(err) = seed_embed_origin_titles().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
