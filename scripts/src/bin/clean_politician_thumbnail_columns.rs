use colored::*;
use reqwest::StatusCode;
use spinners::{Spinner, Spinners};
use std::error::Error;
use std::process;
use std::time::Instant;

async fn clean_politician_thumbnails() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let mut sp = Spinner::new(Spinners::Dots5, "Cleaning".into());
    db::init_pool().await.unwrap();
    let pool = db::pool().await;

    let politicians = db::Politician::index(&pool.connection).await?;
    for politician in politicians {
        if let Some(thumbnail_url) = politician.thumbnail_image_url {
            let response = reqwest::get(&thumbnail_url).await;
            if let Ok(response) = response {
                println!("{}: {}", politician.slug, response.status());
                if response.status() == StatusCode::OK {
                    continue;
                }
            }
            let _ = sqlx::query!(
                r#"
                UPDATE politician
                SET thumbnail_image_url = NULL,
                assets = '{}'::jsonb
                WHERE id = $1
                "#,
                politician.id
            )
            .execute(&pool.connection)
            .await?;
        } else {
            continue;
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
    if let Err(err) = clean_politician_thumbnails().await {
        println!("Error occurred: {}", err);
        process::exit(1);
    }
}
