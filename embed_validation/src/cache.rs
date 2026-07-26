use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::{Duration, Instant};

/// In-memory cache of host-page fetches, keyed by normalized URL.
///
/// Multiple embed pings for the same page within the TTL share one HTTP GET.
#[derive(Debug)]
pub struct PageFetchCache {
    ttl: Duration,
    entries: RwLock<HashMap<String, CacheEntry>>,
}

#[derive(Debug, Clone)]
enum CacheEntry {
    Html { body: Arc<str>, stored_at: Instant },
    NotFound { stored_at: Instant },
}

/// Result of fetching (or reading from cache) a host page's HTML.
#[derive(Debug, Clone)]
pub enum PageFetchResult {
    Html(Arc<str>),
    NotFound,
    FetchFailed { message: String },
}

impl PageFetchCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: RwLock::new(HashMap::new()),
        }
    }

    fn is_fresh(&self, stored_at: Instant) -> bool {
        stored_at.elapsed() < self.ttl
    }

    fn get_if_fresh(&self, url: &str) -> Option<PageFetchResult> {
        let guard = self.entries.read().ok()?;
        let entry = guard.get(url)?;
        match entry {
            CacheEntry::Html { body, stored_at } if self.is_fresh(*stored_at) => {
                Some(PageFetchResult::Html(Arc::clone(body)))
            }
            CacheEntry::NotFound { stored_at } if self.is_fresh(*stored_at) => {
                Some(PageFetchResult::NotFound)
            }
            _ => None,
        }
    }

    fn store(&self, url: &str, result: &PageFetchResult) {
        let Some(mut guard) = self.entries.write().ok() else {
            return;
        };
        match result {
            PageFetchResult::Html(body) => {
                guard.insert(
                    url.to_string(),
                    CacheEntry::Html {
                        body: Arc::clone(body),
                        stored_at: Instant::now(),
                    },
                );
            }
            PageFetchResult::NotFound => {
                guard.insert(
                    url.to_string(),
                    CacheEntry::NotFound {
                        stored_at: Instant::now(),
                    },
                );
            }
            PageFetchResult::FetchFailed { .. } => {
                // Do not cache transient failures so the next ping can retry.
            }
        }
    }

    /// Returns cached HTML / 404 when fresh; otherwise fetches and updates the cache.
    pub async fn fetch_page(&self, client: &reqwest::Client, url: &str) -> PageFetchResult {
        if let Some(cached) = self.get_if_fresh(url) {
            return cached;
        }

        let result = super::fetch_page_uncached(client, url).await;

        if !matches!(result, PageFetchResult::FetchFailed { .. }) {
            self.store(url, &result);
        }

        result
    }

    #[cfg(test)]
    pub(crate) fn insert_html_for_test(&self, url: &str, html: &str) {
        let mut guard = self.entries.write().unwrap();
        guard.insert(
            url.to_string(),
            CacheEntry::Html {
                body: Arc::from(html),
                stored_at: Instant::now(),
            },
        );
    }
}

static GLOBAL_PAGE_CACHE: OnceLock<PageFetchCache> = OnceLock::new();

/// TTL for [`global_page_fetch_cache`], from `EMBED_ORIGIN_CACHE_TTL_SECS` (default 60).
pub fn page_fetch_cache_ttl() -> Duration {
    std::env::var("EMBED_ORIGIN_CACHE_TTL_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(60))
}

/// Process-wide page cache used by `ping_embed_origin`.
pub fn global_page_fetch_cache() -> &'static PageFetchCache {
    GLOBAL_PAGE_CACHE.get_or_init(|| PageFetchCache::new(page_fetch_cache_ttl()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_entry_is_returned_without_fetch() {
        let cache = PageFetchCache::new(Duration::from_secs(60));
        cache.insert_html_for_test("https://example.com/article", "<html></html>");

        let result = cache.get_if_fresh("https://example.com/article").unwrap();
        assert!(matches!(result, PageFetchResult::Html(_)));
    }

    #[test]
    fn expired_entry_is_not_returned() {
        let cache = PageFetchCache::new(Duration::from_millis(0));
        cache.insert_html_for_test("https://example.com/article", "<html></html>");
        std::thread::sleep(Duration::from_millis(1));

        assert!(cache.get_if_fresh("https://example.com/article").is_none());
    }
}
