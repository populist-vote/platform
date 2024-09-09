use std::error::Error;

use regex::Regex;
use sqlx::{postgres::PgListener, PgPool};

pub async fn listener(db_pool: PgPool) -> Result<(), Box<dyn Error>> {
    // Create a PgListener
    let mut listener = PgListener::connect_with(&db_pool).await?;

    // Start listening to the new_embed_origin channel
    listener.listen("new_embed_origin").await?;

    // Continuously receive notifications
    loop {
        // Wait for a notification
        let notification = listener.recv().await?;
        let url = notification.payload();

        // Fetch and update the title for the received URL
        if let Err(e) = fetch_and_update_title(&url, &db_pool).await {
            eprintln!("Failed to fetch or update the title for {}: {}", url, e);
        }
    }
}

async fn fetch_and_update_title(
    url: &str,
    db_pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Fetch the HTML content of the page
    let html = reqwest::get(url).await?.text().await?;

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
    .execute(db_pool)
    .await?;

    Ok(())
}
