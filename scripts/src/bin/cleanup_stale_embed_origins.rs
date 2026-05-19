use colored::*;
use embed_validation::{
    default_http_client, page_fetch_cache_ttl, verify_embed_on_page_cached, PageFetchCache,
    VerificationMode, VerifyResult,
};
use indicatif::{ProgressBar, ProgressStyle};
use std::error::Error;
use std::process;
use std::time::Instant;

struct EmbedOrigin {
    embed_id: uuid::Uuid,
    url: String,
    page_title: Option<String>,
}

enum CheckResult {
    Valid(Option<String>),
    NotFound,
    EmbedNotPresent,
}

async fn cleanup_stale_embed_origins(dry_run: bool, verbose: bool) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    println!(
        "\n{} {}\n",
        "🔍".bold(),
        if dry_run {
            "Scanning embed origins (DRY RUN - no deletions will occur)"
                .bright_yellow()
                .bold()
        } else {
            "Scanning and cleaning up stale embed origins"
                .bright_cyan()
                .bold()
        }
    );

    db::init_pool().await.unwrap();
    let db_pool = db::pool().await;

    let origins = sqlx::query_as!(
        EmbedOrigin,
        r#"
        SELECT embed_id, url, page_title
        FROM embed_origin
        ORDER BY url
        "#
    )
    .fetch_all(&db_pool.connection)
    .await?;

    let total_count = origins.len();
    println!("📊 Found {} embed origin records to check\n", total_count);

    if total_count == 0 {
        println!("✅ No records to process");
        return Ok(());
    }

    let pb = ProgressBar::new(total_count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    let client = default_http_client()?;
    let page_cache = PageFetchCache::new(page_fetch_cache_ttl());

    let mut valid_count = 0;
    let mut invalid_count = 0;
    let mut error_count = 0;
    let mut not_found_count = 0;
    let mut title_updated_count = 0;
    let mut deleted_urls: Vec<String> = Vec::new();
    let mut not_found_urls: Vec<String> = Vec::new();

    for origin in origins {
        pb.inc(1);

        match check_embed_exists(&client, &page_cache, &origin.url, &origin.embed_id).await {
            Ok(CheckResult::Valid(page_title)) => {
                valid_count += 1;

                let should_update = match (&page_title, &origin.page_title) {
                    (Some(new_title), Some(old_title)) => new_title != old_title,
                    (Some(_), None) => true,
                    (None, Some(_)) => false,
                    (None, None) => false,
                };

                if should_update {
                    title_updated_count += 1;

                    if verbose {
                        println!(
                            "\n📝 {} title for {}",
                            if dry_run { "Would update" } else { "Updating" },
                            origin.url
                        );
                        println!("   Old: {:?}", origin.page_title);
                        println!("   New: {:?}", page_title);
                    }

                    if !dry_run {
                        match sqlx::query!(
                            r#"
                            UPDATE embed_origin
                            SET page_title = $1
                            WHERE embed_id = $2 AND url = $3
                            "#,
                            page_title,
                            origin.embed_id,
                            origin.url
                        )
                        .execute(&db_pool.connection)
                        .await
                        {
                            Ok(result) => {
                                if verbose {
                                    println!("   ✅ Updated {} row(s)", result.rows_affected());
                                }
                            }
                            Err(e) => {
                                eprintln!("\n⚠️  Failed to update title for {}: {}", origin.url, e);
                            }
                        }
                    }
                }
            }
            Ok(CheckResult::NotFound) => {
                not_found_count += 1;
                not_found_urls.push(origin.url.clone());

                if !dry_run {
                    match sqlx::query!(
                        r#"
                        DELETE FROM embed_origin
                        WHERE embed_id = $1 AND url = $2
                        "#,
                        origin.embed_id,
                        origin.url
                    )
                    .execute(&db_pool.connection)
                    .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("\n❌ Failed to delete {}: {}", origin.url, e);
                            error_count += 1;
                        }
                    }
                }
            }
            Ok(CheckResult::EmbedNotPresent) => {
                invalid_count += 1;
                deleted_urls.push(origin.url.clone());

                if !dry_run {
                    match sqlx::query!(
                        r#"
                        DELETE FROM embed_origin
                        WHERE embed_id = $1 AND url = $2
                        "#,
                        origin.embed_id,
                        origin.url
                    )
                    .execute(&db_pool.connection)
                    .await
                    {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("\n❌ Failed to delete {}: {}", origin.url, e);
                            error_count += 1;
                        }
                    }
                }
            }
            Err(e) => {
                error_count += 1;
                eprintln!("\n⚠️  Error checking {}: {}", origin.url, e);
            }
        }
    }

    pb.finish_with_message("Done!");

    println!("\n{}", "═".repeat(60));
    println!("{}", "Summary".bright_white().bold());
    println!("{}", "═".repeat(60));
    println!(
        "✅ Valid embeds:        {} ({:.1}%)",
        valid_count.to_string().bright_green().bold(),
        (valid_count as f64 / total_count as f64 * 100.0)
    );
    println!(
        "❌ Stale embeds:        {} ({:.1}%)",
        invalid_count.to_string().bright_red().bold(),
        (invalid_count as f64 / total_count as f64 * 100.0)
    );
    println!(
        "🚫 Pages not found:     {} ({:.1}%)",
        not_found_count.to_string().bright_magenta().bold(),
        (not_found_count as f64 / total_count as f64 * 100.0)
    );
    println!(
        "⚠️  Other errors:       {}",
        error_count.to_string().bright_yellow().bold()
    );
    let total_to_delete = invalid_count + not_found_count;
    println!(
        "\n📊 Total to delete:     {} ({:.1}%)",
        total_to_delete.to_string().bright_cyan().bold(),
        (total_to_delete as f64 / total_count as f64 * 100.0)
    );
    if title_updated_count > 0 {
        if dry_run {
            println!(
                "📝 Titles to update:    {}",
                title_updated_count.to_string().bright_blue().bold()
            );
        } else {
            println!(
                "📝 Titles updated:      {}",
                title_updated_count.to_string().bright_blue().bold()
            );
        }
    }
    println!("{}", "═".repeat(60));

    if dry_run && (invalid_count > 0 || not_found_count > 0) {
        if invalid_count > 0 {
            println!(
                "\n{}",
                "URLs where embed is not present (would be deleted):"
                    .bright_yellow()
                    .bold()
            );
            for url in &deleted_urls {
                println!("  • {}", url);
            }
        }
        if not_found_count > 0 {
            println!(
                "\n{}",
                "URLs that return 404 (would be deleted):"
                    .bright_magenta()
                    .bold()
            );
            for url in &not_found_urls {
                println!("  • {}", url);
            }
        }
        println!("\n💡 Run without --dry-run flag to actually delete these records");
    } else if !dry_run && (invalid_count > 0 || not_found_count > 0) {
        println!("\n{} {} records deleted", "🗑️".bold(), total_to_delete);
        if invalid_count > 0 {
            println!("  • {} stale embeds (embed not present)", invalid_count);
        }
        if not_found_count > 0 {
            println!("  • {} pages not found (404)", not_found_count);
        }
    }

    let duration = start.elapsed();
    println!("\n🕑 Completed in {:?}\n", duration);

    Ok(())
}

async fn check_embed_exists(
    client: &reqwest::Client,
    page_cache: &PageFetchCache,
    url: &str,
    embed_id: &uuid::Uuid,
) -> Result<CheckResult, Box<dyn Error>> {
    match verify_embed_on_page_cached(
        client,
        Some(page_cache),
        url,
        embed_id,
        VerificationMode::Lenient,
    )
    .await
    {
        VerifyResult::Present { page_title } => Ok(CheckResult::Valid(page_title)),
        VerifyResult::PageNotFound => Ok(CheckResult::NotFound),
        VerifyResult::NotPresent => Ok(CheckResult::EmbedNotPresent),
        VerifyResult::FetchFailed { message } => Err(message.into()),
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.contains(&"--dry-run".to_string());
    let verbose = args.contains(&"--verbose".to_string()) || args.contains(&"-v".to_string());

    if let Err(err) = cleanup_stale_embed_origins(dry_run, verbose).await {
        eprintln!("\n❌ Error occurred: {}", err);
        process::exit(1);
    }
}
