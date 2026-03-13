//! Merges validated rows from ingest_staging.stg_tx_scraped_us_house_candidates
//! into production politician (campaign_website_url, official_website_url, social URLs,
//! email, thumbnail_image_url). Only rows with validated_at IS NOT NULL and merged_at IS NULL
//! are merged. Plan: docs/tx_house_candidate_web_scrape_plan.md

use sqlx::PgPool;

#[derive(Debug, sqlx::FromRow)]
struct StagingRow {
    politician_id: uuid::Uuid,
    source_url: Option<String>,
    source_type: Option<String>,
    campaign_website_url: Option<String>,
    official_website_url: Option<String>,
    facebook_url: Option<String>,
    twitter_url: Option<String>,
    instagram_url: Option<String>,
    tiktok_url: Option<String>,
    youtube_url: Option<String>,
    linkedin_url: Option<String>,
    email: Option<String>,
    thumbnail_image_url: Option<String>,
}

#[tokio::main]
async fn main() {
    db::init_pool().await.unwrap();
    let pool = db::pool().await;
    let db = &pool.connection;

    println!("=== Merge politician web contacts (stg_tx_scraped_us_house_candidates) → production ===\n");

    match run_merge(db).await {
        Ok(n) => println!("\n✓ Merge completed. Updated {} politician(s).", n),
        Err(e) => {
            eprintln!("Merge failed: {}", e);
            std::process::exit(1);
        }
    }
}

async fn run_merge(pool: &PgPool) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let rows: Vec<StagingRow> = sqlx::query_as(
        r#"
        SELECT politician_id, source_url, source_type, campaign_website_url, official_website_url,
               facebook_url, twitter_url, instagram_url, tiktok_url, youtube_url, linkedin_url,
               email, thumbnail_image_url
        FROM ingest_staging.stg_tx_scraped_us_house_candidates
        WHERE validated_at IS NOT NULL AND merged_at IS NULL
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut updated = 0u64;
    for row in &rows {
        sqlx::query(
            r#"
            UPDATE politician
            SET
                campaign_website_url = COALESCE($2, campaign_website_url),
                official_website_url = COALESCE($3, official_website_url),
                facebook_url = COALESCE($4, facebook_url),
                twitter_url = COALESCE($5, twitter_url),
                instagram_url = COALESCE($6, instagram_url),
                tiktok_url = COALESCE($7, tiktok_url),
                youtube_url = COALESCE($8, youtube_url),
                linkedin_url = COALESCE($9, linkedin_url),
                email = COALESCE($10, email),
                thumbnail_image_url = COALESCE($11, thumbnail_image_url),
                updated_at = (now() AT TIME ZONE 'utc')
            WHERE id = $1
            "#,
        )
        .bind(row.politician_id)
        .bind(&row.campaign_website_url)
        .bind(&row.official_website_url)
        .bind(&row.facebook_url)
        .bind(&row.twitter_url)
        .bind(&row.instagram_url)
        .bind(&row.tiktok_url)
        .bind(&row.youtube_url)
        .bind(&row.linkedin_url)
        .bind(&row.email)
        .bind(&row.thumbnail_image_url)
        .execute(pool)
        .await?;

        sqlx::query(
            r#"
            UPDATE ingest_staging.stg_tx_scraped_us_house_candidates
            SET merged_at = (now() AT TIME ZONE 'utc')
            WHERE politician_id = $1
            "#,
        )
        .bind(row.politician_id)
        .execute(pool)
        .await?;

        updated += 1;
    }

    Ok(updated)
}
