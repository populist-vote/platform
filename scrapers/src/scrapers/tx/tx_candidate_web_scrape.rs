//! TX U.S. House candidate web scrape: discovers campaign/official URLs via search,
//! scrapes campaign or Ballotpedia pages for social links, email, profile image;
//! writes to ingest_staging.stg_tx_scraped_us_house_candidates.
//!
//! Plan: docs/tx_house_candidate_web_scrape_plan.md

use regex::Regex;
use scraper::{Html, Selector};
use sqlx::PgPool;
use std::time::Duration;
use url::Url;

const ELECTION_ID_TX_MARCH3_PRIMARY: &str = "0d586931-c119-4fe7-814f-f679e91282a8";
const REQUEST_DELAY_MS: u64 = 2000;
const USER_AGENT: &str = "Mozilla/5.0 (compatible; PopulistCandidateScraper/1.0; +https://populist.us)";

/// Words that must appear in the URL or page title to consider a site a campaign site. Easy to extend.
const CAMPAIGN_SITE_KEYWORDS: &[&str] = &["congress", "texas"];

/// Row returned by the candidate query (TX U.S. House, March 3 primary).
#[derive(Debug, sqlx::FromRow)]
pub struct CandidateRow {
    pub id: uuid::Uuid,
    pub slug: String,
    pub first_name: String,
    pub last_name: String,
    pub preferred_name: Option<String>,
    pub full_name: Option<String>,
    pub office_id: Option<uuid::Uuid>,
    pub campaign_website_url: Option<String>,
    pub official_website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub youtube_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub tiktok_url: Option<String>,
    pub email: Option<String>,
    pub thumbnail_image_url: Option<String>,
    /// From office.name (current office held); used for search queries.
    pub current_office_name: Option<String>,
    pub race_title: Option<String>,
}

/// Staging row for stg_tx_scraped_us_house_candidates.
#[derive(Debug)]
pub struct StagingRow {
    pub politician_id: uuid::Uuid,
    pub politician_full_name: Option<String>,
    pub source_url: Option<String>,
    pub source_type: Option<String>,
    pub campaign_website_url: Option<String>,
    pub official_website_url: Option<String>,
    pub facebook_url: Option<String>,
    pub twitter_url: Option<String>,
    pub instagram_url: Option<String>,
    pub tiktok_url: Option<String>,
    pub youtube_url: Option<String>,
    pub linkedin_url: Option<String>,
    pub email: Option<String>,
    pub thumbnail_image_url: Option<String>,
    pub scrape_status: String,
}

fn has_manual_data(c: &CandidateRow) -> bool {
    let has = |s: &Option<String>| s.as_ref().map_or(false, |t| !t.trim().is_empty());
    has(&c.facebook_url)
        || has(&c.twitter_url)
        || has(&c.instagram_url)
        || has(&c.youtube_url)
        || has(&c.linkedin_url)
        || has(&c.tiktok_url)
}

/// Create ingest_staging schema and stg_tx_scraped_us_house_candidates table.
pub async fn ensure_staging_table(pool: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ingest_staging.stg_tx_scraped_us_house_candidates (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            politician_id UUID NOT NULL UNIQUE,
            politician_full_name TEXT,
            source_url TEXT,
            source_type TEXT,
            campaign_website_url TEXT,
            official_website_url TEXT,
            facebook_url TEXT,
            twitter_url TEXT,
            instagram_url TEXT,
            tiktok_url TEXT,
            youtube_url TEXT,
            linkedin_url TEXT,
            email TEXT,
            thumbnail_image_url TEXT,
            scraped_at TIMESTAMPTZ NOT NULL DEFAULT (now() AT TIME ZONE 'utc'),
            scrape_status TEXT NOT NULL,
            validated_at TIMESTAMPTZ,
            merged_at TIMESTAMPTZ
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

/// Load TX U.S. House candidates for the March 3 primary (excludes rows with manual data in code; DB returns all).
pub async fn load_candidates(pool: &PgPool) -> Result<Vec<CandidateRow>, Box<dyn std::error::Error + Send + Sync>> {
    let rows = sqlx::query_as::<_, CandidateRow>(
        r#"
        SELECT DISTINCT
            p.id,
            p.slug,
            p.first_name,
            p.last_name,
            p.preferred_name,
            p.full_name,
            p.office_id,
            p.campaign_website_url,
            p.official_website_url,
            p.facebook_url,
            p.twitter_url,
            p.instagram_url,
            p.youtube_url,
            p.linkedin_url,
            p.tiktok_url,
            p.email,
            p.thumbnail_image_url,
            office_curr.name AS current_office_name,
            r.title AS race_title
        FROM politician p
        JOIN race_candidates rc ON rc.candidate_id = p.id
        JOIN race r ON r.id = rc.race_id
        JOIN office o ON o.id = r.office_id
        LEFT JOIN office office_curr ON p.office_id = office_curr.id
        WHERE p.home_state = 'TX'::state
          AND o.title = 'U.S. Representative'
          AND r.election_id = $1
        "#,
    )
    .bind(uuid::Uuid::parse_str(ELECTION_ID_TX_MARCH3_PRIMARY).unwrap())
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

/// Build DuckDuckGo HTML search URL (first iteration: scrape search result pages).
fn duckduckgo_search_url(query: &str) -> String {
    let encoded = form_urlencoded::byte_serialize(query.as_bytes()).collect::<String>();
    format!("https://html.duckduckgo.com/html/?q={}", encoded)
}

/// Fetch HTML with a polite User-Agent and timeout.
async fn fetch_html(client: &reqwest::Client, url: &str) -> Result<String, reqwest::Error> {
    let resp = client
        .get(url)
        .header("User-Agent", USER_AGENT)
        .timeout(Duration::from_secs(15))
        .send()
        .await?;
    let resp = resp.error_for_status()?;
    resp.text().await
}

/// Parse DuckDuckGo HTML results and return first few result URLs (excluding DDG/duckduckgo).
fn parse_duckduckgo_results(html: &str) -> Vec<String> {
    let doc = Html::parse_document(html);
    let mut urls = Vec::new();
    if let Ok(sel) = Selector::parse("a.result__a") {
        for el in doc.select(&sel) {
            if let Some(href) = el.value().attr("href") {
                if let Ok(u) = Url::parse(href) {
                    let host = u.host_str().unwrap_or("");
                    if !host.contains("duckduckgo") && !host.is_empty() {
                        urls.push(u.to_string());
                    }
                }
            }
        }
    }
    if urls.is_empty() {
        if let Ok(sel) = Selector::parse("a.result__url") {
            for el in doc.select(&sel) {
                if let Some(href) = el.value().attr("href") {
                    if let Ok(u) = Url::parse(href) {
                        let host = u.host_str().unwrap_or("");
                        if !host.contains("duckduckgo") && !host.is_empty() {
                            urls.push(u.to_string());
                        }
                    }
                }
            }
        }
    }
    urls
}

/// Extracts the document title from HTML (first `<title>`). Logs URL and title to stderr for campaign-site checks.
fn extract_page_title(html: &str, url: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse("title").ok()?;
    let title = doc
        .select(&sel)
        .next()
        .map(|el| el.inner_html().trim().to_string());
    eprintln!(
        "[campaign check] url: {} | title: {}",
        url,
        title.as_deref().unwrap_or("(none)")
    );
    title
}

/// Heuristic: does this URL (and optional page title) look like a campaign site?
/// Excludes social, ballotpedia, house.gov. When name parts are provided, requires the URL or title
/// to contain at least one name part and at least one keyword from CAMPAIGN_SITE_KEYWORDS.
fn looks_like_campaign_site(
    url: &str,
    title: Option<&str>,
    first_name: Option<&str>,
    last_name: Option<&str>,
    preferred_name: Option<&str>,
) -> bool {
    let url_lower = url.to_lowercase();
    if url_lower.contains("facebook.com") || url_lower.contains("twitter.com") || url_lower.contains("x.com")
        || url_lower.contains("instagram.com") || url_lower.contains("tiktok.com")
        || url_lower.contains("youtube.com") || url_lower.contains("linkedin.com")
        || url_lower.contains("ballotpedia.org")
        || url_lower.contains("house.gov") || url_lower.contains("senate.gov")
    {
        return false;
    }
    let combined = format!(
        "{} {}",
        url_lower,
        title.map(|t| t.to_lowercase()).unwrap_or_default()
    );
    let name_parts: Vec<String> = [first_name, last_name, preferred_name]
        .into_iter()
        .flatten()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    if !name_parts.is_empty() {
        let has_name = name_parts.iter().any(|p| combined.contains(p));
        let has_keyword = CAMPAIGN_SITE_KEYWORDS
            .iter()
            .any(|kw| combined.contains(*kw));
        if !has_name || !has_keyword {
            return false;
        }
    }
    true
}

/// Heuristic: does this URL look like an official government site?
fn looks_like_official_site(url: &str) -> bool {
    let lower = url.to_lowercase();
    lower.contains("house.gov") || lower.contains("senate.gov") || lower.contains(".gov")
}

/// Heuristic: does this URL look like a Ballotpedia candidate page?
fn looks_like_ballotpedia_candidate(url: &str) -> bool {
    let lower = url.to_lowercase();
    lower.contains("ballotpedia.org") && (lower.contains("/wiki/") || lower.contains("Ballotpedia"))
}

/// Search for a URL (campaign or official or Ballotpedia) via DuckDuckGo; returns first plausible URL.
async fn search_first_result<F>(client: &reqwest::Client, query: &str, filter: F) -> Result<Option<String>, reqwest::Error>
where
    F: Fn(&str) -> bool,
{
    let url = duckduckgo_search_url(query);
    let html = fetch_html(client, &url).await?;
    for u in parse_duckduckgo_results(&html) {
        if filter(&u) {
            return Ok(Some(u));
        }
    }
    Ok(None)
}

/// URL-only check: exclude social, gov, ballotpedia so we don't fetch them when looking for campaign sites.
fn url_fails_campaign_exclude(url: &str) -> bool {
    let lower = url.to_lowercase();
    lower.contains("facebook.com") || lower.contains("twitter.com") || lower.contains("x.com")
        || lower.contains("instagram.com") || lower.contains("tiktok.com")
        || lower.contains("youtube.com") || lower.contains("linkedin.com")
        || lower.contains("ballotpedia.org")
        || lower.contains("house.gov") || lower.contains("senate.gov")
}

/// Search DuckDuckGo for a campaign site, then fetch each candidate URL and check URL + page title
/// for name parts and CAMPAIGN_SITE_KEYWORDS. Returns the first URL that passes.
async fn search_first_campaign_site(
    client: &reqwest::Client,
    query: &str,
    first_name: &str,
    last_name: &str,
    preferred_name: Option<&str>,
    delay: Duration,
) -> Result<Option<String>, reqwest::Error> {
    let search_url = duckduckgo_search_url(query);
    let html = fetch_html(client, &search_url).await?;
    for url in parse_duckduckgo_results(&html) {
        eprintln!("url testing: {}", url);
        if url_fails_campaign_exclude(&url) {
            continue;
        }
        tokio::time::sleep(delay).await;
        let page_html = match fetch_html(client, &url).await {
            Ok(h) => h,
            Err(_) => continue,
        };
        let title = extract_page_title(&page_html, &url);
        eprintln!("title of url tested: {}", title.as_deref().unwrap_or("(none)"));
        if looks_like_campaign_site(
            &url,
            title.as_deref(),
            Some(first_name),
            Some(last_name),
            preferred_name,
        ) {
            return Ok(Some(url));
        }
    }
    Ok(None)
}

/// Extract all <a href="..."> and <meta property="og:image"> from HTML; normalize and filter social links.
fn extract_social_and_email(html: &str, base_url: &str) -> (Vec<(String, String)>, Option<String>) {
    let base = Url::parse(base_url).unwrap_or_else(|_| Url::parse("https://example.com").unwrap());
    let doc = Html::parse_document(html);
    let mut facebook = None;
    let mut twitter = None;
    let mut instagram = None;
    let mut tiktok = None;
    let mut youtube = None;
    let mut linkedin = None;
    let mut email = None;

    let link_selector = Selector::parse("a[href]").unwrap_or_else(|_| unreachable!());
    for el in doc.select(&link_selector) {
        let href = match el.value().attr("href") {
            Some(h) => h.trim(),
            None => continue,
        };
        if href.is_empty() {
            continue;
        }
        let full = base.join(href).ok().map(|u| u.to_string()).unwrap_or_else(|| href.to_string());
        let lower = full.to_lowercase();
        if lower.contains("mailto:") {
            let addr = href.strip_prefix("mailto:").unwrap_or(href).split_whitespace().next().unwrap_or("");
            if !addr.is_empty() && addr.contains('@') {
                email = email.or(Some(addr.to_string()));
            }
            continue;
        }
        if lower.contains("facebook.com") {
            facebook = facebook.or(Some(normalize_social_url(&full, "facebook")));
        } else if lower.contains("twitter.com") || lower.contains("x.com") {
            twitter = twitter.or(Some(normalize_social_url(&full, "twitter")));
        } else if lower.contains("instagram.com") {
            instagram = instagram.or(Some(normalize_social_url(&full, "instagram")));
        } else if lower.contains("tiktok.com") {
            tiktok = tiktok.or(Some(normalize_social_url(&full, "tiktok")));
        } else if lower.contains("youtube.com") {
            youtube = youtube.or(Some(normalize_social_url(&full, "youtube")));
        } else if lower.contains("linkedin.com") {
            linkedin = linkedin.or(Some(normalize_social_url(&full, "linkedin")));
        }
    }

    if email.is_none() {
        let re = Regex::new(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}").unwrap();
        if let Some(cap) = re.find(html) {
            email = Some(cap.as_str().to_string());
        }
    }

    let mut social = Vec::new();
    if let Some(u) = facebook {
        social.push(("facebook_url".to_string(), u));
    }
    if let Some(u) = twitter {
        social.push(("twitter_url".to_string(), u));
    }
    if let Some(u) = instagram {
        social.push(("instagram_url".to_string(), u));
    }
    if let Some(u) = tiktok {
        social.push(("tiktok_url".to_string(), u));
    }
    if let Some(u) = youtube {
        social.push(("youtube_url".to_string(), u));
    }
    if let Some(u) = linkedin {
        social.push(("linkedin_url".to_string(), u));
    }
    (social, email)
}

fn normalize_social_url(url: &str, _platform: &str) -> String {
    let u = match Url::parse(url) {
        Ok(u) => u,
        Err(_) => return url.to_string(),
    };
    let mut s = format!("https://{}", u.host_str().unwrap_or(""));
    let p = u.path();
    if p != "/" && !p.is_empty() {
        s.push_str(p);
    }
    if u.query().is_some() {
        s.push('?');
        s.push_str(u.query().unwrap());
    }
    s
}

/// Prefer og:image; else first large-enough img src (not icon/logo).
///
/// **Facebook profile image (when facebook_url was found):** See docs/tx_house_candidate_web_scrape_plan.md §4.1.
/// Plan: when we have a scraped `facebook_url`, fetch that page, parse `<meta property="og:image">`, optionally
/// upgrade the CDN URL for higher resolution (_s → _o or _n), and use that for thumbnail_image_url (e.g. via
/// a separate async `fetch_facebook_profile_image_url(client, facebook_url)` called in the main loop after
/// extracting socials, with rate limiting).
fn extract_profile_image(html: &str, base_url: &str) -> Option<String> {
    let base = Url::parse(base_url).unwrap_or_else(|_| Url::parse("https://example.com").unwrap());
    let doc = Html::parse_document(html);

    let meta_selector = Selector::parse(r#"meta[property="og:image"]"#).unwrap_or_else(|_| unreachable!());
    for el in doc.select(&meta_selector) {
        if let Some(c) = el.value().attr("content") {
            let full = base.join(c.trim()).ok().map(|u| u.to_string()).unwrap_or_else(|| c.to_string());
            if !full.is_empty() {
                return Some(full);
            }
        }
    }

    let img_selector = Selector::parse("img[src]").unwrap_or_else(|_| unreachable!());
    for el in doc.select(&img_selector) {
        let src = el.value().attr("src").unwrap_or("");
        let lower = src.to_lowercase();
        if lower.contains("logo") || lower.contains("icon") || lower.contains("button") || lower.contains("sprite") {
            continue;
        }
        let full = base.join(src.trim()).ok().map(|u| u.to_string()).unwrap_or_else(|| src.to_string());
        if !full.is_empty() {
            return Some(full);
        }
    }
    None
}

/// From Ballotpedia page: get "Campaign website" or "Website" link if present.
fn extract_campaign_website_from_ballotpedia(html: &str, base_url: &str) -> Option<String> {
    let base = Url::parse(base_url).unwrap_or_else(|_| Url::parse("https://example.com").unwrap());
    let doc = Html::parse_document(html);
    let link_selector = Selector::parse("a[href]").unwrap_or_else(|_| unreachable!());
    for el in doc.select(&link_selector) {
        let href = el.value().attr("href")?;
        let text = el.text().collect::<String>().to_lowercase();
        if (text.contains("campaign website") || text.contains("website")) && !href.starts_with("#") {
            let full = base.join(href.trim()).ok().map(|u| u.to_string())?;
            if !full.contains("ballotpedia.org") {
                return Some(full);
            }
        }
    }
    None
}

/// Upsert one row into ingest_staging.stg_tx_scraped_us_house_candidates.
pub async fn upsert_staging_row(
    pool: &PgPool,
    row: &StagingRow,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query(
        r#"
        INSERT INTO ingest_staging.stg_tx_scraped_us_house_candidates (
            politician_id, politician_full_name, source_url, source_type, campaign_website_url, official_website_url,
            facebook_url, twitter_url, instagram_url, tiktok_url, youtube_url, linkedin_url,
            email, thumbnail_image_url, scrape_status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        ON CONFLICT (politician_id) DO UPDATE SET
            politician_full_name = EXCLUDED.politician_full_name,
            source_url = EXCLUDED.source_url,
            source_type = EXCLUDED.source_type,
            campaign_website_url = EXCLUDED.campaign_website_url,
            official_website_url = EXCLUDED.official_website_url,
            facebook_url = EXCLUDED.facebook_url,
            twitter_url = EXCLUDED.twitter_url,
            instagram_url = EXCLUDED.instagram_url,
            tiktok_url = EXCLUDED.tiktok_url,
            youtube_url = EXCLUDED.youtube_url,
            linkedin_url = EXCLUDED.linkedin_url,
            email = EXCLUDED.email,
            thumbnail_image_url = EXCLUDED.thumbnail_image_url,
            scraped_at = (now() AT TIME ZONE 'utc'),
            scrape_status = EXCLUDED.scrape_status
        "#,
    )
    .bind(row.politician_id)
    .bind(&row.politician_full_name)
    .bind(&row.source_url)
    .bind(&row.source_type)
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
    .bind(&row.scrape_status)
    .execute(pool)
    .await?;
    Ok(())
}

/// Run the full scrape: load candidates, for each (skip manual, search official, resolve scrape URL, fetch, extract, write staging).
/// If `limit` is `Some(n)`, only the first `n` candidates are scraped (useful for testing or incremental runs).
pub async fn run(pool: &PgPool, limit: Option<usize>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    ensure_staging_table(pool).await?;
    let mut candidates = load_candidates(pool).await?;
    if let Some(n) = limit {
        candidates.truncate(n);
    }

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(15))
        .build()?;

    let delay = Duration::from_millis(REQUEST_DELAY_MS);
    let total = candidates.len();
    eprintln!("Processing {} candidate(s)\n", total);
    for (i, c) in candidates.iter().enumerate() {
        eprintln!("[{}/{}] {}", i + 1, total, c.full_name.as_deref().unwrap_or(&c.slug));

        if has_manual_data(c) {
            eprintln!("  → skip: already has manual data (campaign/social URLs set)");
            let row = StagingRow {
                politician_id: c.id,
                politician_full_name: c.full_name.clone(),
                source_url: None,
                source_type: None,
                campaign_website_url: None,
                official_website_url: None,
                facebook_url: None,
                twitter_url: None,
                instagram_url: None,
                tiktok_url: None,
                youtube_url: None,
                linkedin_url: None,
                email: None,
                thumbnail_image_url: None,
                scrape_status: "skipped_manual_data".to_string(),
            };
            upsert_staging_row(pool, &row).await?;
            continue;
        }

        let mut official_website_url: Option<String> = None;
        if c.office_id.is_some() {
            let office_name = c.current_office_name.as_deref().unwrap_or("U.S. House");
            let name = c.full_name.as_deref().unwrap_or("candidate");
            let query = format!("{} {} Texas official website", name, office_name);
            eprintln!("  → official site search: {}", query);
            tokio::time::sleep(delay).await;
            match search_first_result(&client, &query, looks_like_official_site).await {
                Ok(Some(url)) => {
                    eprintln!("  → official site found: {}", url);
                    official_website_url = Some(url);
                }
                _ => eprintln!("  → official site: (none)"),
            }
        }

        let (source_url, source_type, campaign_website_url) = if let Some(ref u) = c.campaign_website_url {
            if !u.trim().is_empty() {
                eprintln!("  → source: DB campaign_site: {}", u);
                (Some(u.clone()), Some("campaign_site".to_string()), Some(u.clone()))
            } else {
                (None, None, None)
            }
        } else {
            (None, None, None)
        };

        let (source_url, source_type, campaign_website_url) = if source_url.is_some() {
            (source_url, source_type, campaign_website_url)
        } else {
            let name = c.full_name.as_deref().unwrap_or("candidate");
            let race_title = c.race_title.as_deref().unwrap_or("TX U.S. House");
            let query = format!("{} {} candidate campaign website", name, race_title);
            eprintln!("  → campaign site search: {}", query);
            tokio::time::sleep(delay).await;
            match search_first_campaign_site(
                &client,
                &query,
                &c.first_name,
                &c.last_name,
                c.preferred_name.as_deref(),
                delay,
            )
            .await
            {
                Ok(Some(url)) => {
                    eprintln!("  → campaign site found: {}", url);
                    (Some(url.clone()), Some("web_search_campaign".to_string()), Some(url))
                }
                _ => {
                    let query_ballot = format!("{} ballotpedia", name);
                    eprintln!("  → campaign site: (none), trying Ballotpedia: {}", query_ballot);
                    tokio::time::sleep(delay).await;
                    match search_first_result(&client, &query_ballot, looks_like_ballotpedia_candidate).await {
                        Ok(Some(url)) => {
                            eprintln!("  → ballotpedia found: {}", url);
                            (Some(url.clone()), Some("ballotpedia".to_string()), None)
                        }
                        _ => {
                            eprintln!("  → ballotpedia: (none)");
                            (None, None, None)
                        }
                    }
                }
            }
        };

        if source_url.is_none() {
            eprintln!("  → no source URL → staging: no_site");
            let row = StagingRow {
                politician_id: c.id,
                politician_full_name: c.full_name.clone(),
                source_url: None,
                source_type: None,
                campaign_website_url: None,
                official_website_url,
                facebook_url: None,
                twitter_url: None,
                instagram_url: None,
                tiktok_url: None,
                youtube_url: None,
                linkedin_url: None,
                email: None,
                thumbnail_image_url: None,
                scrape_status: "no_site".to_string(),
            };
            upsert_staging_row(pool, &row).await?;
            continue;
        }

        let source_url = source_url.unwrap();
        let source_type = source_type.unwrap_or_else(|| "unknown".to_string());
        eprintln!("  → fetch: {} (source_type={})", source_url, source_type);
        tokio::time::sleep(delay).await;

        let html = match fetch_html(&client, &source_url).await {
            Ok(h) => h,
            Err(e) => {
                eprintln!("  → fetch failed: {} → staging: fetch_error", e);
                let row = StagingRow {
                    politician_id: c.id,
                    politician_full_name: c.full_name.clone(),
                    source_url: Some(source_url),
                    source_type: Some(source_type),
                    campaign_website_url,
                    official_website_url,
                    facebook_url: None,
                    twitter_url: None,
                    instagram_url: None,
                    tiktok_url: None,
                    youtube_url: None,
                    linkedin_url: None,
                    email: None,
                    thumbnail_image_url: None,
                    scrape_status: "fetch_error".to_string(),
                };
                upsert_staging_row(pool, &row).await?;
                continue;
            }
        };

        let (socials, email) = extract_social_and_email(&html, &source_url);
        let mut facebook_url = None;
        let mut twitter_url = None;
        let mut instagram_url = None;
        let mut tiktok_url = None;
        let mut youtube_url = None;
        let mut linkedin_url = None;
        for (k, v) in socials {
            match k.as_str() {
                "facebook_url" => facebook_url = Some(v),
                "twitter_url" => twitter_url = Some(v),
                "instagram_url" => instagram_url = Some(v),
                "tiktok_url" => tiktok_url = Some(v),
                "youtube_url" => youtube_url = Some(v),
                "linkedin_url" => linkedin_url = Some(v),
                _ => {}
            }
        }

        let campaign_website_url = campaign_website_url
            .or_else(|| extract_campaign_website_from_ballotpedia(&html, &source_url));
        let thumbnail_image_url = extract_profile_image(&html, &source_url);

        eprintln!(
            "  → extracted: campaign={} email={} thumbnail={} fb={} twitter={} ig={}",
            campaign_website_url.as_deref().unwrap_or("(none)"),
            if email.is_some() { "yes" } else { "no" },
            if thumbnail_image_url.is_some() { "yes" } else { "no" },
            if facebook_url.is_some() { "yes" } else { "no" },
            if twitter_url.is_some() { "yes" } else { "no" },
            if instagram_url.is_some() { "yes" } else { "no" },
        );
        eprintln!("  → staging: ok");
        let row = StagingRow {
            politician_id: c.id,
            politician_full_name: c.full_name.clone(),
            source_url: Some(source_url),
            source_type: Some(source_type),
            campaign_website_url,
            official_website_url,
            facebook_url,
            twitter_url,
            instagram_url,
            tiktok_url,
            youtube_url,
            linkedin_url,
            email,
            thumbnail_image_url,
            scrape_status: "ok".to_string(),
        };
        upsert_staging_row(pool, &row).await?;
    }

    Ok(())
}
