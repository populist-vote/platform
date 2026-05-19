//! Validates that a Populist embed ID appears in a host page's HTML.
//!
//! Used by `ping_embed_origin` (strict mode) and `cleanup_stale_embed_origins` (lenient mode).

mod cache;

use std::time::Duration;
use uuid::Uuid;

pub use cache::{
    global_page_fetch_cache, page_fetch_cache_ttl, PageFetchCache, PageFetchResult,
};

use regex::Regex;

const USER_AGENT: &str = "Mozilla/5.0 (compatible; PopulistBot/1.0; +https://populist.us)";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationMode {
    /// Require high-confidence markup (widget container, iframe, or script data-embed-id).
    Strict,
    /// Broader heuristics for cleanup jobs (matches legacy cleanup script behavior).
    Lenient,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyResult {
    Present { page_title: Option<String> },
    NotPresent,
    PageNotFound,
    FetchFailed { message: String },
}

/// Shared HTTP client for host-page fetches (10s timeout, Populist user-agent).
pub fn default_http_client() -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(USER_AGENT)
        .build()
}

/// Fetches `url` (using the process-wide cache) and checks whether `embed_id` is in the HTML.
pub async fn verify_embed_on_page(
    client: &reqwest::Client,
    url: &str,
    embed_id: &Uuid,
    mode: VerificationMode,
) -> VerifyResult {
    verify_embed_on_page_cached(
        client,
        Some(global_page_fetch_cache()),
        url,
        embed_id,
        mode,
    )
    .await
}

/// Fetches `url` with an optional per-request cache, then checks `embed_id` in the HTML.
pub async fn verify_embed_on_page_cached(
    client: &reqwest::Client,
    page_cache: Option<&PageFetchCache>,
    url: &str,
    embed_id: &Uuid,
    mode: VerificationMode,
) -> VerifyResult {
    let fetch_result = match page_cache {
        Some(cache) => cache.fetch_page(client, url).await,
        None => fetch_page_uncached(client, url).await,
    };

    match fetch_result {
        PageFetchResult::Html(html) => verify_embed_in_html(&html, embed_id, mode),
        PageFetchResult::NotFound => VerifyResult::PageNotFound,
        PageFetchResult::FetchFailed { message } => VerifyResult::FetchFailed { message },
    }
}

/// Checks whether `embed_id` appears in already-fetched HTML.
pub fn verify_embed_in_html(html: &str, embed_id: &Uuid, mode: VerificationMode) -> VerifyResult {
    let page_title = extract_page_title(html);
    let embed_id_str = embed_id.to_string();

    if embed_present_in_html(html, &embed_id_str, mode) {
        VerifyResult::Present { page_title }
    } else {
        VerifyResult::NotPresent
    }
}

async fn fetch_page_uncached(client: &reqwest::Client, url: &str) -> PageFetchResult {
    let response = match client.get(url).send().await {
        Ok(r) => r,
        Err(e) => {
            return PageFetchResult::FetchFailed {
                message: e.to_string(),
            };
        }
    };

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return PageFetchResult::NotFound;
    }

    if !response.status().is_success() {
        return PageFetchResult::FetchFailed {
            message: format!("HTTP {}", response.status()),
        };
    }

    match response.text().await {
        Ok(html) => PageFetchResult::Html(html.into()),
        Err(e) => PageFetchResult::FetchFailed {
            message: e.to_string(),
        },
    }
}

pub fn extract_page_title(html: &str) -> Option<String> {
    let head_regex = Regex::new(r"(?is)<head[^>]*>(.*?)</head>").unwrap();
    let head_content = head_regex
        .captures(html)
        .and_then(|caps| caps.get(1).map(|m| m.as_str()))?;

    let title_regex = Regex::new(r"(?i)<title[^>]*>(.*?)</title>").unwrap();
    if let Some(caps) = title_regex.captures(head_content) {
        if let Some(title_text) = caps.get(1).map(|m| m.as_str().trim().to_string()) {
            if !title_text.is_empty() {
                return Some(title_text);
            }
        }
    }

    let meta_title_exact_regex = Regex::new(
        r#"(?i)<meta[^>]*name=["']title["'][^>]*content=["']([^"']*)["'][^>]*>|<meta[^>]*content=["']([^"']*)["'][^>]*name=["']title["'][^>]*>"#,
    )
    .unwrap();

    if let Some(caps) = meta_title_exact_regex.captures(head_content) {
        let content = caps
            .get(1)
            .or_else(|| caps.get(2))
            .map(|m| m.as_str().trim().to_string());

        if let Some(title) = content {
            if !title.is_empty() {
                return Some(title);
            }
        }
    }

    let meta_title_regex = Regex::new(
        r#"(?i)<meta[^>]*(?:name|property)=["']([^"']*title[^"']*)["'][^>]*content=["']([^"']*)["'][^>]*>|<meta[^>]*content=["']([^"']*)["'][^>]*(?:name|property)=["']([^"']*title[^"']*)["'][^>]*>"#,
    )
    .unwrap();

    if let Some(caps) = meta_title_regex.captures(head_content) {
        let content = caps
            .get(2)
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

fn embed_present_in_html(html: &str, embed_id: &str, mode: VerificationMode) -> bool {
    match mode {
        VerificationMode::Strict => strict_embed_present(html, embed_id),
        VerificationMode::Lenient => lenient_embed_present(html, embed_id),
    }
}

fn strict_embed_present(html: &str, embed_id: &str) -> bool {
    let escaped = regex::escape(embed_id);

    let v2_container = Regex::new(&format!(
        r#"(?is)<[a-z0-9]+[^>]*class=["'][^"']*populist-embed[^"']*["'][^>]*data-embed-id=["']{escaped}["']"#,
    ))
    .unwrap();
    if v2_container.is_match(html) {
        return true;
    }

    let v2_container_rev = Regex::new(&format!(
        r#"(?is)<[a-z0-9]+[^>]*data-embed-id=["']{escaped}["'][^>]*class=["'][^"']*populist-embed[^"']*["']"#,
    ))
    .unwrap();
    if v2_container_rev.is_match(html) {
        return true;
    }

    let script_embed_id = Regex::new(&format!(
        r#"(?is)<script[^>]*data-embed-id=["']{escaped}["']"#,
    ))
    .unwrap();
    if script_embed_id.is_match(html) {
        return true;
    }

    let v1_container = Regex::new(&format!(
        r#"(?is)<[a-z0-9]+[^>]*class=["'][^"']*populist-{escaped}[^"']*["']"#,
    ))
    .unwrap();
    if v1_container.is_match(html) {
        return true;
    }

    let iframe = Regex::new(&format!(
        r#"(?is)<iframe[^>]*(?:/embeds/{escaped}|embeds%2F{escaped})"#,
    ))
    .unwrap();
    if iframe.is_match(html) {
        return true;
    }

    let data_embed_id = Regex::new(&format!(r#"data-embed-id=["']{escaped}["']"#)).unwrap();
    if data_embed_id.is_match(html)
        && (html.contains("widget-client-v2.js")
            || html.contains("widget-client.js")
            || html.contains("populist-embed"))
    {
        return true;
    }

    false
}

fn lenient_embed_present(html: &str, embed_id: &str) -> bool {
    if strict_embed_present(html, embed_id) {
        return true;
    }

    if html.contains(embed_id) {
        return true;
    }

    let escaped = regex::escape(embed_id);

    let script_pattern = Regex::new(&format!(
        r#"(?i)(populist.*embed|embed.*populist).*{escaped}|{escaped}.*(?:populist.*embed|embed.*populist)"#,
    ))
    .unwrap();
    if script_pattern.is_match(html) {
        return true;
    }

    let iframe_pattern = Regex::new(&format!(r#"(?is)<iframe[^>]*{escaped}[^>]*>"#)).unwrap();
    if iframe_pattern.is_match(html) {
        return true;
    }

    let data_attr_pattern =
        Regex::new(&format!(r#"data-[^=]*=["']?[^"']*{escaped}[^"']*["']?"#)).unwrap();
    if data_attr_pattern.is_match(html) {
        return true;
    }

    let div_pattern = Regex::new(&format!(
        r#"(?is)<div[^>]*(?:class=["'][^"']*populist[^"']*["']|id=["'][^"']*populist[^"']*["'])[^>]*>[^<]*{escaped}|<div[^>]*>[^<]*{escaped}[^<]*(?:class=["'][^"']*populist[^"']*["']|id=["'][^"']*populist[^"']*["'])"#,
    ))
    .unwrap();
    if div_pattern.is_match(html) {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    const EMBED_ID: &str = "086b0f58-6cf1-4d89-bef8-56c7cbd6a606";

    #[test]
    fn strict_accepts_widget_v2_container() {
        let html = r#"
        <div class="populist-embed" data-embed-id="086b0f58-6cf1-4d89-bef8-56c7cbd6a606"></div>
        <script src="https://www.populist.us/widget-client-v2.js"></script>
        "#;
        assert!(strict_embed_present(html, EMBED_ID));
    }

    #[test]
    fn strict_accepts_iframe_embed() {
        let html = r#"<iframe src="https://www.populist.us/embeds/086b0f58-6cf1-4d89-bef8-56c7cbd6a606?origin=https://example.com"></iframe>"#;
        assert!(strict_embed_present(html, EMBED_ID));
    }

    #[test]
    fn strict_rejects_bare_uuid_in_json() {
        let html = r#"<script>window.config = {"embedId": "086b0f58-6cf1-4d89-bef8-56c7cbd6a606"};</script>"#;
        assert!(!strict_embed_present(html, EMBED_ID));
    }

    #[test]
    fn lenient_accepts_bare_uuid() {
        let html = r#"<script>window.config = {"id": "086b0f58-6cf1-4d89-bef8-56c7cbd6a606"};</script>"#;
        assert!(lenient_embed_present(html, EMBED_ID));
    }

    #[test]
    fn strict_rejects_different_embed_id() {
        let html =
            r#"<div class="populist-embed" data-embed-id="00000000-0000-0000-0000-000000000001"></div>"#;
        assert!(!strict_embed_present(html, EMBED_ID));
    }

    #[test]
    fn extract_page_title_from_head() {
        let html = "<html><head><title>Texas Runoffs</title></head><body></body></html>";
        assert_eq!(
            extract_page_title(html),
            Some("Texas Runoffs".to_string())
        );
    }

    #[test]
    fn embed_present_respects_mode() {
        let html = r#"<p>086b0f58-6cf1-4d89-bef8-56c7cbd6a606</p>"#;
        assert!(!strict_embed_present(html, EMBED_ID));
        assert!(lenient_embed_present(html, EMBED_ID));
    }

    #[test]
    fn verify_in_html_checks_multiple_embeds_without_fetch() {
        let html = r#"
        <div class="populist-embed" data-embed-id="086b0f58-6cf1-4d89-bef8-56c7cbd6a606"></div>
        <div class="populist-embed" data-embed-id="00000000-0000-0000-0000-000000000001"></div>
        "#;
        let id_a = Uuid::parse_str("086b0f58-6cf1-4d89-bef8-56c7cbd6a606").unwrap();
        let id_b = Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap();

        assert!(matches!(
            verify_embed_in_html(html, &id_a, VerificationMode::Strict),
            VerifyResult::Present { .. }
        ));
        assert!(matches!(
            verify_embed_in_html(html, &id_b, VerificationMode::Strict),
            VerifyResult::Present { .. }
        ));
    }
}
