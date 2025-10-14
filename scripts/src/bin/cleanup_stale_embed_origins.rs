use colored::*;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use std::error::Error;
use std::process;
use std::time::Instant;

struct EmbedOrigin {
    embed_id: uuid::Uuid,
    url: String,
    page_title: Option<String>,
}

async fn cleanup_stale_embed_origins(dry_run: bool, verbose: bool) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    
    println!(
        "\n{} {}\n",
        "🔍".bold(),
        if dry_run {
            "Scanning embed origins (DRY RUN - no deletions will occur)".bright_yellow().bold()
        } else {
            "Scanning and cleaning up stale embed origins".bright_cyan().bold()
        }
    );

    db::init_pool().await.unwrap();
    let db_pool = db::pool().await;

    // Fetch all embed origins
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

    // Create progress bar
    let pb = ProgressBar::new(total_count as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (compatible; PopulistBot/1.0; +https://populist.us)")
        .build()?;

    let mut valid_count = 0;
    let mut invalid_count = 0;
    let mut error_count = 0;
    let mut not_found_count = 0;
    let mut title_updated_count = 0;
    let mut deleted_urls: Vec<String> = Vec::new();
    let mut not_found_urls: Vec<String> = Vec::new();

    for origin in origins {
        pb.inc(1);
        
        // Check if the embed is still present on the page
        match check_embed_exists(&client, &origin.url, &origin.embed_id).await {
            Ok(CheckResult::Valid(page_title)) => {
                valid_count += 1;
                
                // Update page title if it's different from what we have or if we don't have one
                let should_update = match (&page_title, &origin.page_title) {
                    (Some(new_title), Some(old_title)) => new_title != old_title,
                    (Some(_), None) => true, // We have a new title but didn't have one before
                    (None, Some(_)) => false, // Don't overwrite existing title with None
                    (None, None) => false, // Both are None, no update needed
                };
                
                if should_update {
                    title_updated_count += 1;
                    
                    if verbose {
                        println!("\n📝 {} title for {}", 
                            if dry_run { "Would update" } else { "Updating" },
                            origin.url
                        );
                        println!("   Old: {:?}", origin.page_title);
                        println!("   New: {:?}", page_title);
                    }
                    
                    // Only update title in production mode, not dry-run
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
                            },
                            Err(e) => {
                                eprintln!("\n⚠️  Failed to update title for {}: {}", origin.url, e);
                            }
                        }
                    }
                }
            }
            Ok(CheckResult::NotFound) => {
                // Page doesn't exist (404) - delete the record
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
                        Ok(_) => {},
                        Err(e) => {
                            eprintln!("\n❌ Failed to delete {}: {}", origin.url, e);
                            error_count += 1;
                        }
                    }
                }
            }
            Ok(CheckResult::EmbedNotPresent) => {
                // Page exists but embed is not present - delete the record
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
                        Ok(_) => {},
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

    // Print summary
    println!("\n{}", "═".repeat(60));
    println!("{}", "Summary".bright_white().bold());
    println!("{}", "═".repeat(60));
    println!("✅ Valid embeds:        {} ({:.1}%)", 
        valid_count.to_string().bright_green().bold(),
        (valid_count as f64 / total_count as f64 * 100.0)
    );
    println!("❌ Stale embeds:        {} ({:.1}%)", 
        invalid_count.to_string().bright_red().bold(),
        (invalid_count as f64 / total_count as f64 * 100.0)
    );
    println!("🚫 Pages not found:     {} ({:.1}%)", 
        not_found_count.to_string().bright_magenta().bold(),
        (not_found_count as f64 / total_count as f64 * 100.0)
    );
    println!("⚠️  Other errors:       {}", 
        error_count.to_string().bright_yellow().bold()
    );
    let total_to_delete = invalid_count + not_found_count;
    println!("\n📊 Total to delete:     {} ({:.1}%)", 
        total_to_delete.to_string().bright_cyan().bold(),
        (total_to_delete as f64 / total_count as f64 * 100.0)
    );
    if title_updated_count > 0 {
        if dry_run {
            println!("📝 Titles to update:    {}", 
                title_updated_count.to_string().bright_blue().bold()
            );
        } else {
            println!("📝 Titles updated:      {}", 
                title_updated_count.to_string().bright_blue().bold()
            );
        }
    }
    println!("{}", "═".repeat(60));

    if dry_run && (invalid_count > 0 || not_found_count > 0) {
        if invalid_count > 0 {
            println!("\n{}", "URLs where embed is not present (would be deleted):".bright_yellow().bold());
            for url in &deleted_urls {
                println!("  • {}", url);
            }
        }
        if not_found_count > 0 {
            println!("\n{}", "URLs that return 404 (would be deleted):".bright_magenta().bold());
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

enum CheckResult {
    Valid(Option<String>), // Contains the page title
    NotFound,
    EmbedNotPresent,
}

fn extract_page_title(html: &str) -> Option<String> {
    // Extract only the <head> section
    let head_regex = Regex::new(r"(?is)<head[^>]*>(.*?)</head>").unwrap();
    let head_content = head_regex
        .captures(html)
        .and_then(|caps| caps.get(1).map(|m| m.as_str()))?; // Return None if no <head> found

    // Priority 1: Look for <title> tag within <head>
    let title_regex = Regex::new(r"(?i)<title[^>]*>(.*?)</title>").unwrap();
    if let Some(caps) = title_regex.captures(head_content) {
        if let Some(title_text) = caps.get(1).map(|m| m.as_str().trim().to_string()) {
            if !title_text.is_empty() {
                return Some(title_text);
            }
        }
    }

    // Priority 2: Look for meta tag with name="title"
    let meta_title_exact_regex = Regex::new(
        r#"(?i)<meta[^>]*name=["']title["'][^>]*content=["']([^"']*)["'][^>]*>|<meta[^>]*content=["']([^"']*)["'][^>]*name=["']title["'][^>]*>"#
    ).unwrap();
    
    if let Some(caps) = meta_title_exact_regex.captures(head_content) {
        let content = caps.get(1)
            .or_else(|| caps.get(2))
            .map(|m| m.as_str().trim().to_string());
        
        if let Some(title) = content {
            if !title.is_empty() {
                return Some(title);
            }
        }
    }

    // Priority 3: Look for other meta tags with "title" in the name property
    // This includes og:title, twitter:title, etc.
    let meta_title_regex = Regex::new(
        r#"(?i)<meta[^>]*(?:name|property)=["']([^"']*title[^"']*)["'][^>]*content=["']([^"']*)["'][^>]*>|<meta[^>]*content=["']([^"']*)["'][^>]*(?:name|property)=["']([^"']*title[^"']*)["'][^>]*>"#
    ).unwrap();
    
    if let Some(caps) = meta_title_regex.captures(head_content) {
        // The content might be in group 2 or group 3 depending on attribute order
        let content = caps.get(2)
            .or_else(|| caps.get(3))
            .map(|m| m.as_str().trim().to_string());
        
        if let Some(title) = content {
            if !title.is_empty() {
                return Some(title);
            }
        }
    }

    None
}

async fn check_embed_exists(
    client: &reqwest::Client,
    url: &str,
    embed_id: &uuid::Uuid,
) -> Result<CheckResult, Box<dyn Error>> {
    // Fetch the HTML content
    let response = client.get(url).send().await?;
    
    // Check if page exists (404 means page is gone)
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(CheckResult::NotFound);
    }
    
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()).into());
    }

    let html = response.text().await?;
    
    // Extract page title from HTML
    let page_title = extract_page_title(&html);
    
    // Check for various patterns that indicate the embed is present
    let embed_id_str = embed_id.to_string();
    
    // Pattern 1: Direct embed ID in script tag or data attribute
    if html.contains(&embed_id_str) {
        return Ok(CheckResult::Valid(page_title));
    }
    
    // Pattern 2: Check for populist embed script with the ID
    let script_pattern = Regex::new(&format!(
        r#"(?i)(populist.*embed|embed.*populist).*{}|{}.*(?:populist.*embed|embed.*populist)"#,
        regex::escape(&embed_id_str),
        regex::escape(&embed_id_str)
    ))?;
    
    if script_pattern.is_match(&html) {
        return Ok(CheckResult::Valid(page_title));
    }
    
    // Pattern 3: Check for iframe with embed ID
    let iframe_pattern = Regex::new(&format!(
        r#"<iframe[^>]*{}[^>]*>"#,
        regex::escape(&embed_id_str)
    ))?;
    
    if iframe_pattern.is_match(&html) {
        return Ok(CheckResult::Valid(page_title));
    }
    
    // Pattern 4: Check for data attributes with embed ID
    let data_attr_pattern = Regex::new(&format!(
        r#"data-[^=]*=["']?[^"']*{}[^"']*["']?"#,
        regex::escape(&embed_id_str)
    ))?;
    
    if data_attr_pattern.is_match(&html) {
        return Ok(CheckResult::Valid(page_title));
    }
    
    // Pattern 5: Check for div with populist embed class and ID
    let div_pattern = Regex::new(&format!(
        r#"<div[^>]*(?:class=["'][^"']*populist[^"']*["']|id=["'][^"']*populist[^"']*["'])[^>]*>[^<]*{}|<div[^>]*>[^<]*{}[^<]*(?:class=["'][^"']*populist[^"']*["']|id=["'][^"']*populist[^"']*["'])"#,
        regex::escape(&embed_id_str),
        regex::escape(&embed_id_str)
    ))?;
    
    if div_pattern.is_match(&html) {
        return Ok(CheckResult::Valid(page_title));
    }
    
    // If none of the patterns match, the embed is not present
    Ok(CheckResult::EmbedNotPresent)
}

#[tokio::main]
async fn main() {
    // Check for flags
    let args: Vec<String> = std::env::args().collect();
    let dry_run = args.contains(&"--dry-run".to_string());
    let verbose = args.contains(&"--verbose".to_string()) || args.contains(&"-v".to_string());

    if let Err(err) = cleanup_stale_embed_origins(dry_run, verbose).await {
        eprintln!("\n❌ Error occurred: {}", err);
        process::exit(1);
    }
}

