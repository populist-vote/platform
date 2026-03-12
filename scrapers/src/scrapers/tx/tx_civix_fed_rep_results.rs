//! Scrape Texas Republican and Democratic primary race results from the Civix Election Night Results page.
//!
//! Flow: one driver session per election. Session 1: load page → modal → select Republican → scrape all tabs.
//! Session 2 (new driver): load page → modal → select Democratic → scrape all tabs (default Federal Offices tab).
//! Requires chromedriver running (e.g. `chromedriver --port=9515`). Writes rows to
//! `ingest_staging.stg_tx_results_sos_civix` (Race, Choice, Party, early_votes, votes_for_candidate, total_votes, vote_pct).

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

use csv::WriterBuilder;
use once_cell::sync::Lazy;
use regex::Regex;
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use thirtyfour::prelude::*;

use crate::generators::politician::PoliticianRefKeyGenerator;

static RE_NORMALIZE_RACE_STAR: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)star_rate").unwrap());
static RE_NORMALIZE_RACE_CLICK: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?i)click for contest details").unwrap());

/// Set to true to print debug info to stderr (options count, table row count, etc.).
const DEBUG_SCRAPE: bool = true;

const CIVIX_RACES_URL: &str = "https://goelect.txelections.civixapps.com/ivis-enr-ui/races";
const TX_SOS_DATA_DIR: &str = "data/tx/sos";
const OUTPUT_CSV: &str = "sos-results-fed-rep.csv";

#[allow(dead_code)]
fn debug(s: &str) {
    if DEBUG_SCRAPE {
        eprintln!("[civix] {}", s);
    }
}

/// One row for the output CSV / staging table.
#[derive(Debug, Clone)]
pub struct ResultRow {
    pub ref_key: String,
    pub race: String,
    pub choice: String,
    pub party: String,
    pub early_votes: u64,
    pub votes_for_candidate: u64,
    pub total_votes: u64,
    pub vote_pct: String,
}

pub fn output_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    std::path::Path::new(&manifest_dir)
        .join(TX_SOS_DATA_DIR)
        .join(OUTPUT_CSV)
}

fn parse_votes(s: &str) -> u64 {
    s.replace(',', "").trim().parse().unwrap_or(0)
}

/// Normalize choice (candidate name): remove "(I)" and trim whitespace.
fn normalize_choice(s: &str) -> String {
    s.replace("(I)", "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// Normalize race text: remove "star_rate", "click for contest details" (case-insensitive), then trim whitespace.
fn normalize_race(s: &str) -> String {
    let s = RE_NORMALIZE_RACE_STAR.replace_all(s, "");
    let s = RE_NORMALIZE_RACE_CLICK.replace_all(s.as_ref(), "");
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

/// Count mat-option elements via WebDriver (no script return value — avoids serialization issues).
async fn count_mat_options(driver: &WebDriver) -> usize {
    driver
        .find_all(By::Css("mat-option"))
        .await
        .map(|els| els.len())
        .unwrap_or(0)
}

/// Get option texts by finding mat-option elements via WebDriver and reading .text() (no script return).
async fn get_open_dropdown_options_via_elements(
    driver: &WebDriver,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let opts = driver.find_all(By::Css("mat-option")).await?;
    let mut texts = Vec::with_capacity(opts.len());
    for el in opts {
        let t = el.text().await.unwrap_or_default();
        let t = t.trim();
        if !t.is_empty() {
            texts.push(t.to_string());
        }
    }
    Ok(texts)
}

/// Return option texts from the currently open dropdown panel.
/// Uses WebDriver find_all("mat-option") and element.text() so we don't depend on script return values.
/// Polls until options appear (panel may render shortly after the dropdown opens).
async fn get_open_dropdown_options(
    driver: &WebDriver,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    let poll_interval = Duration::from_millis(200);
    let timeout = Duration::from_secs(3);
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if let Ok(opts) = get_open_dropdown_options_via_elements(driver).await {
            if !opts.is_empty() {
                return Ok(opts);
            }
        }
        tokio::time::sleep(poll_interval).await;
    }
    if DEBUG_SCRAPE {
        eprintln!("[civix] get_open_dropdown_options: no mat-option elements found after {:?} (dropdown may not have opened)", timeout);
    }
    Ok(Vec::new())
}

/// Select the given election in the modal. Finds the mat-form-field whose text contains `dropdown_identifier`
/// (case-insensitive) — e.g. "Select Election" on first open, or "2026 republican primary election" when reopening after a selection.
/// Clicks it to open the dropdown, selects `option_to_select`, then "Update Election".
async fn select_election_in_modal(
    driver: &WebDriver,
    dropdown_identifier: &str,
    option_to_select: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ident_lower = dropdown_identifier.trim().to_lowercase();
    let form_field_selector = "mat-form-field.mat-mdc-form-field-type-mat-select";
    let elems = match driver.find_all(By::Css(form_field_selector)).await {
        Ok(e) => e,
        Err(_) => {
            debug("modal: no mat-form-field.mat-mdc-form-field-type-mat-select found, trying mat-form-field");
            driver.find_all(By::Css("mat-form-field")).await.unwrap_or_default()
        }
    };
    debug(&format!("modal: {} mat-form-field(s)", elems.len()));

    let mut election_dropdown = None;
    for (i, form_field) in elems.iter().enumerate() {
        let label = form_field
            .text()
            .await
            .unwrap_or_else(|_| String::new())
            .trim()
            .replace('\n', " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        let label_lower = label.to_lowercase();
        let label_display = if label.is_empty() { "?" } else { label.as_str() };
        debug(&format!("form-field {} (\"{}\")", i, label_display));
        if label_lower.contains(&ident_lower) {
            election_dropdown = Some((i, form_field));
            break;
        }
    }

    let (i, form_field) = match election_dropdown {
        Some(p) => p,
        None => {
            debug(&format!("no mat-form-field with \"{}\" found", dropdown_identifier));
            return Ok(());
        }
    };

    debug(&format!("opening dropdown (form-field index {}, identifier \"{}\")", i, dropdown_identifier));

    let elem_value = match form_field.to_json() {
        Ok(v) => v,
        Err(e) => {
            debug(&format!("to_json for mat-form-field failed: {}", e));
            return Ok(());
        }
    };
    let describe_el_js = r#"
        var el = arguments[0];
        if (!el) return 'element null';
        var tag = el.tagName ? el.tagName.toLowerCase() : '?';
        var cls = (el.className && typeof el.className === 'string') ? el.className : '';
        return tag + ' class=' + cls.substring(0, 100);
    "#;
    if let Ok(v) = driver.execute(describe_el_js, vec![elem_value.clone()]).await {
        let desc: String = v.convert::<Option<String>>().unwrap_or(None).unwrap_or_else(|| "?".into());
        debug(&format!("element to click: {}", desc));
    }

    // Use WebDriver find_all so we don't depend on script return values.
    let before = count_mat_options(driver).await;
    debug(&format!("mat-option count before click: {}", before));

    // Click the mat-form-field (or the select trigger inside it). MDC uses .mat-mdc-select-trigger or the inner mat-select.
    let click_form_field_js = r#"
        var el = arguments[0];
        if (!el) return 'no element';
        el.scrollIntoView({ block: 'center', behavior: 'instant' });
        var trigger = el.querySelector && (
            el.querySelector('.mat-mdc-select-trigger') ||
            el.querySelector('.mat-select-trigger') ||
            el.querySelector('mat-select') ||
            el.querySelector('[role="combobox"]') ||
            el.querySelector('.mat-mdc-form-field-flex')
        );
        var toClick = (trigger && trigger.offsetParent !== null) ? trigger : el;
        toClick.click();
        return 'clicked';
    "#;
    if driver
        .execute(click_form_field_js, vec![elem_value])
        .await
        .is_err()
    {
        debug("execute click on mat-form-field (or trigger) failed");
        return Ok(());
    }
    debug("click executed on mat-form-field / trigger");

    // Wait for the overlay panel to render.
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Post-click: mat-option count via WebDriver (if dropdown opened, this should be > 0).
    let after = count_mat_options(driver).await;
    debug(&format!("mat-option count after click: {}", after));
    if after == 0 && DEBUG_SCRAPE {
        eprintln!("[civix] dropdown did not open (mat-option count still 0 after click)");
    }

    let options = match get_open_dropdown_options(driver).await {
        Ok(o) => o,
        Err(_) => {
            debug("get options failed");
            return Ok(());
        }
    };

    if DEBUG_SCRAPE {
        let preview: String = options.iter().take(5).cloned().collect::<Vec<_>>().join(" | ");
        let more = if options.len() > 5 { " ..." } else { "" };
        debug(&format!("options ({}): {}{}", options.len(), preview, more));
    }

    if click_option_by_text(driver, option_to_select).await? {
        debug(&format!("selected \"{}\"", option_to_select));
        click_update_election(driver).await?;
    } else {
        let _ = form_field.click().await;
        debug(&format!("option \"{}\" not found in dropdown", option_to_select));
    }
    Ok(())
}

/// Normalize string for case-insensitive comparison: trim and collapse whitespace.
fn normalize_for_match(s: &str) -> String {
    s.trim()
        .replace('\n', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Click an option (mat-option or [role=option]) whose text matches the given text (case-insensitive, whitespace normalized). Returns true if clicked.
async fn click_option_by_text(
    driver: &WebDriver,
    text: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let target = normalize_for_match(text);
    let opts = driver.find_all(By::Css("mat-option")).await?;
    for el in opts {
        let option_text = el.text().await.unwrap_or_default();
        if normalize_for_match(&option_text).eq_ignore_ascii_case(&target) {
            el.click().await?;
            return Ok(true);
        }
    }
    let opts = driver.find_all(By::Css("[role='option']")).await.unwrap_or_default();
    for el in opts {
        let option_text = el.text().await.unwrap_or_default();
        if normalize_for_match(&option_text).eq_ignore_ascii_case(&target) {
            el.click().await?;
            return Ok(true);
        }
    }
    Ok(false)
}

/// Click "Update Election" to apply the selection and load results (modal closes, page loads with race="All", Federal Offices tab).
async fn click_update_election(driver: &WebDriver) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Prefer exact "Update Election" so we don't click a different Update button.
    let xpaths = [
        "//button[contains(normalize-space(.), 'Update Election')]",
        "//*[contains(@class, 'mat-mdc-dialog-actions')]//button[contains(normalize-space(.), 'Update')]",
        "//*[@mat-dialog-actions]//button[contains(normalize-space(.), 'Update')]",
        "//button[contains(normalize-space(.), 'Update')]",
        "//button[contains(normalize-space(.), 'Apply')]",
        "//button[contains(normalize-space(.), 'OK')]",
    ];
    for xpath in &xpaths {
        if let Ok(btn) = driver.find(By::XPath(*xpath)).await {
            btn.click().await?;
            debug("clicked Update Election");
            tokio::time::sleep(Duration::from_millis(300)).await;
            return Ok(());
        }
    }
    debug("Update Election button not found");
    Ok(())
}

/// Election option we want in the initial modal (Republican Federal primary).
const ELECTION_REPUBLICAN_PRIMARY: &str = "2026 Republican Primary Election";

/// Election option for the Democratic primary (selected after switching from Republican).
const ELECTION_DEMOCRATIC_PRIMARY: &str = "2026 Democratic Primary Election";

/// Text used to find the election dropdown in the modal: first time it shows the placeholder/label.
const DROPDOWN_IDENTIFIER_FIRST: &str = "Select Election";

/// Tab labels to scrape in order (Federal Offices is default after load).
const TAB_LABELS: &[&str] = &[
    "Federal Offices",
    "Statewide Offices",
    "District Offices",
    "Statewide Propositions",
];

/// Capture visible app-race-office elements into table_rows with deduplication by (race, choice).
async fn capture_visible_race_offices(
    driver: &WebDriver,
    table_rows: &mut Vec<Vec<String>>,
    seen_race_choice: &mut HashSet<(String, String)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let office_elems = driver.find_all(By::Css("app-race-office")).await.unwrap_or_default();
    for office_el in office_elems {
        let race_name = match office_el.find(By::Css("mat-card > *")).await {
            Ok(el) => el.text().await.unwrap_or_default(),
            Err(_) => String::new(),
        };
        let race_name = race_name.trim().to_string();
        if race_name.is_empty() {
            continue;
        }
        let tr_elems = match office_el.find_all(By::Css("mat-card table tbody tr")).await {
            Ok(els) => els,
            Err(_) => continue,
        };
        for tr_el in tr_elems {
            let cell_elems = match tr_el.find_all(By::Css("td, th")).await {
                Ok(els) => els,
                Err(_) => continue,
            };
            let mut cell_texts: Vec<String> = Vec::with_capacity(cell_elems.len() + 1);
            cell_texts.push(race_name.clone());
            for cell in cell_elems {
                if let Ok(t) = cell.text().await {
                    cell_texts.push(t.trim().to_string());
                }
            }
            if cell_texts.len() > 2 {
                let choice = normalize_choice(cell_texts.get(1).map(|s| s.as_str()).unwrap_or(""));
                let key = (normalize_race(&race_name), choice);
                if seen_race_choice.insert(key) {
                    table_rows.push(cell_texts);
                }
            }
        }
    }
    Ok(())
}

/// Parse collected table_rows (each row = [race_name, td0, td1, ...]) into ResultRows with total_votes applied.
fn parse_table_rows_to_result_rows(table_rows: &[Vec<String>]) -> Vec<ResultRow> {
    let mut race_totals: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
    let mut rows = Vec::new();
    for row in table_rows {
        if row.len() <= 2 {
            continue;
        }
        let race = normalize_race(&row[0]);
        if race.is_empty() {
            continue;
        }
        if row.len() == 5 && row[2].trim().eq_ignore_ascii_case("Race Total") {
            let total = parse_votes(row.get(3).unwrap_or(&String::new()));
            race_totals.insert(race, total);
            continue;
        }
        if row.len() < 5 {
            continue;
        }
        let choice = normalize_choice(&row[1]);
        if choice.is_empty() {
            continue;
        }
        let party = row[2].trim().to_string();
        let early_votes = parse_votes(row.get(3).unwrap_or(&String::new()));
        let votes_for_candidate = parse_votes(row.get(4).unwrap_or(&String::new()));
        let vote_pct = row.get(5).map(|s| s.trim().to_string()).unwrap_or_default();
        let ref_key = PoliticianRefKeyGenerator::new(
            "tx-primaries",
            2026,
            &race,
            if choice.is_empty() { None } else { Some(choice.as_str()) },
        )
        .generate();
        rows.push(ResultRow {
            ref_key,
            race: race.clone(),
            choice,
            party,
            early_votes,
            votes_for_candidate,
            total_votes: 0,
            vote_pct,
        });
    }
    for r in &mut rows {
        if let Some(&tot) = race_totals.get(&r.race) {
            r.total_votes = tot;
        }
    }
    let races_needing_sum: HashSet<String> = rows.iter().filter(|r| r.total_votes == 0).map(|r| r.race.clone()).collect();
    let sums_by_race: std::collections::HashMap<String, u64> = rows
        .iter()
        .filter(|r| races_needing_sum.contains(&r.race))
        .fold(std::collections::HashMap::new(), |mut m, r| {
            *m.entry(r.race.clone()).or_insert(0) += r.votes_for_candidate;
            m
        });
    for r in &mut rows {
        if r.total_votes == 0 {
            if let Some(&sum) = sums_by_race.get(&r.race) {
                r.total_votes = sum;
            }
        }
    }
    rows
}

/// Normalize tab label for matching: trim and collapse whitespace, lowercase.
fn normalize_tab_text(s: &str) -> String {
    s.trim()
        .replace('\n', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Click the tab whose text contains `label` (case-insensitive).
/// First tries to find a span element that contains the label text and click it; then falls back to [role="tab"].
/// Uses JavaScript scrollIntoView + click for reliability.
async fn click_tab_by_label(
    driver: &WebDriver,
    label: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let label_norm = normalize_tab_text(label);
    let click_js = r#"
        var el = arguments[0];
        if (!el) return 'no element';
        el.scrollIntoView({ block: 'center', behavior: 'instant' });
        el.click();
        return 'clicked';
    "#;

    // Try: find a span that contains the label text inside the tab bar (role=tablist), then click it.
    let span_xpath = format!(
        "//*[@role='tablist']//span[contains(translate(., 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', 'abcdefghijklmnopqrstuvwxyz'), '{}')]",
        label_norm
    );
    if let Ok(span_el) = driver.find(By::XPath(&span_xpath)).await {
        let text = span_el.text().await.unwrap_or_default();
        if normalize_tab_text(&text).contains(&label_norm) {
            if let Ok(span_json) = span_el.to_json() {
                if driver.execute(click_js, vec![span_json]).await.is_ok() {
                    if DEBUG_SCRAPE {
                        eprintln!("[civix] click_tab_by_label: clicked span with text \"{}\"", normalize_tab_text(&text));
                    }
                    tokio::time::sleep(Duration::from_millis(400)).await;
                    return Ok(true);
                }
            }
        }
    }

    // Fallback: [role='tab'] / div[role='tab'] with matching text.
    for selector in &["[role='tab']", "div[role='tab']"] {
        let tabs = match driver.find_all(By::Css(*selector)).await {
            Ok(t) => t,
            Err(_) => continue,
        };
        let tab_count = tabs.len();
        if DEBUG_SCRAPE && tab_count == 0 {
            eprintln!("[civix] click_tab_by_label: selector {} found 0 tabs", selector);
        }
        for tab in tabs {
            let text = tab.text().await.unwrap_or_default();
            let text_norm = normalize_tab_text(&text);
            if DEBUG_SCRAPE && !text_norm.is_empty() {
                eprintln!("[civix] click_tab_by_label: tab text (normalized) = \"{}\"", text_norm);
            }
            if text_norm.contains(&label_norm) {
                let tab_json = match tab.to_json() {
                    Ok(j) => j,
                    Err(_) => {
                        if DEBUG_SCRAPE {
                            eprintln!("[civix] click_tab_by_label: to_json failed for tab");
                        }
                        continue;
                    }
                };
                if driver.execute(click_js, vec![tab_json]).await.is_ok() {
                    tokio::time::sleep(Duration::from_millis(400)).await;
                    return Ok(true);
                }
                if DEBUG_SCRAPE {
                    eprintln!("[civix] click_tab_by_label: JS click failed for tab \"{}\"", text_norm);
                }
            }
        }
        if tab_count > 0 {
            break;
        }
    }
    Ok(false)
}

/// Scrape one election (Republican or Democratic) from a fresh page load.
/// Call with a new WebDriver session so the initial modal appears; then select the given election and scrape all tabs.
///
/// - `is_republican`: if true, select Republican primary and click "Federal Offices" tab first; if false, select Democratic and rely on default Federal Offices tab.
pub async fn scrape_civix_one_election(
    driver: &WebDriver,
    is_republican: bool,
) -> Result<Vec<ResultRow>, Box<dyn std::error::Error + Send + Sync>> {
    let election_name = if is_republican {
        "Republican"
    } else {
        "Democratic"
    };
    debug(&format!("scrape_civix_one_election: {} primary (fresh session)", election_name));

    driver.goto(CIVIX_RACES_URL).await?;
    driver
        .set_page_load_timeout(Duration::from_secs(60))
        .await?;
    tokio::time::sleep(Duration::from_secs(3)).await;

    let option = if is_republican {
        ELECTION_REPUBLICAN_PRIMARY
    } else {
        ELECTION_DEMOCRATIC_PRIMARY
    };
    debug(&format!("selecting election in modal: {}", option));
    select_election_in_modal(driver, DROPDOWN_IDENTIFIER_FIRST, option).await?;

    debug("waiting for results to load...");
    tokio::time::sleep(Duration::from_secs(3)).await;
    if is_republican {
        if click_tab_by_label(driver, "Federal Offices").await? {
            debug("clicked Federal Offices tab to start scraping");
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    } else {
        debug("Dem: relying on default Federal Offices tab, skipping tab click");
    }

    let mut table_rows: Vec<Vec<String>> = Vec::with_capacity(2048);
    let mut seen_race_choice: HashSet<(String, String)> = HashSet::new();
    const VIRTUAL_SCROLL_STEP_PX: i32 = 300;
    const SCROLL_WAIT_MS: u64 = 280;
    const MAX_VIRTUAL_SCROLL_STEPS: u32 = 600;
    let scroll_js = r#"
        var el = arguments[0];
        if (!el) return null;
        el.scrollTop = (el.scrollTop || 0) + arguments[1];
        return el.scrollTop;
    "#;
    let get_pos_js = r#"
        var el = arguments[0];
        if (!el) return null;
        return { scrollHeight: el.scrollHeight, clientHeight: el.clientHeight, scrollTop: el.scrollTop };
    "#;

    for (i, tab_label) in TAB_LABELS.iter().enumerate() {
        let should_click_tab = is_republican || i > 0;
        if should_click_tab {
            if click_tab_by_label(driver, tab_label).await? {
                debug(&format!("clicked tab ({}): {}", election_name, tab_label));
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        let viewport = driver.find(By::Css("cdk-virtual-scroll-viewport")).await.ok();
        let viewport_json = viewport
            .as_ref()
            .and_then(|el| el.to_json().ok())
            .unwrap_or(JsonValue::Null);

        if viewport_json != JsonValue::Null {
            let mut step = 0u32;
            loop {
                let at_bottom = driver
                    .execute(get_pos_js, vec![viewport_json.clone()])
                    .await
                    .ok()
                    .and_then(|v| v.convert::<JsonValue>().ok())
                    .and_then(|j| {
                        let obj = match &j {
                            JsonValue::Object(m) if m.contains_key("scrollHeight") => m,
                            JsonValue::Object(m) => m.get("value").and_then(|v| v.as_object()).unwrap_or(m),
                            _ => return Some(false),
                        };
                        let sh = obj.get("scrollHeight").and_then(|v| v.as_i64()).unwrap_or(0);
                        let ch = obj.get("clientHeight").and_then(|v| v.as_i64()).unwrap_or(0);
                        let st = obj.get("scrollTop").and_then(|v| v.as_i64()).unwrap_or(0);
                        Some(st + ch >= sh && sh > 0)
                    })
                    .unwrap_or(false);

                if at_bottom && step > 0 {
                    debug(&format!("{} ({}): virtual scroll at bottom after {} steps", tab_label, election_name, step));
                    break;
                }
                if step >= MAX_VIRTUAL_SCROLL_STEPS {
                    debug(&format!("{} ({}): virtual scroll hit max steps", tab_label, election_name));
                    break;
                }

                capture_visible_race_offices(driver, &mut table_rows, &mut seen_race_choice).await?;

                let _ = driver
                    .execute(scroll_js, vec![viewport_json.clone(), JsonValue::from(VIRTUAL_SCROLL_STEP_PX)])
                    .await;
                tokio::time::sleep(Duration::from_millis(SCROLL_WAIT_MS)).await;
                step += 1;
            }
        } else {
            let n = table_rows.len();
            capture_visible_race_offices(driver, &mut table_rows, &mut seen_race_choice).await?;
            debug(&format!("{} ({}): no viewport, single pass ({} new rows)", tab_label, election_name, table_rows.len() - n));
        }
    }

    debug(&format!("table_rows count ({}): {}", election_name, table_rows.len()));
    let rows = parse_table_rows_to_result_rows(&table_rows);
    debug(&format!("{} rows parsed: {}", election_name, rows.len()));
    Ok(rows)
}

/// Scrape Republican primary only (single driver session). For Rep + Dem, create two drivers and call `scrape_civix_one_election` twice.
pub async fn scrape_civix_fed_rep_results(
    driver: &WebDriver,
) -> Result<Vec<ResultRow>, Box<dyn std::error::Error + Send + Sync>> {
    scrape_civix_one_election(driver, true).await
}

pub fn write_results_csv(rows: &[ResultRow], path: &std::path::Path) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut wtr = WriterBuilder::new().from_path(path)?;
    wtr.write_record(&["ref_key", "Race", "Choice", "Party", "early_votes", "votes_for_candidate", "total_votes", "vote_pct"])?;
    for r in rows {
        wtr.write_record(&[
            &r.ref_key,
            &r.race,
            &r.choice,
            &r.party,
            &r.early_votes.to_string(),
            &r.votes_for_candidate.to_string(),
            &r.total_votes.to_string(),
            &r.vote_pct,
        ])?;
    }
    wtr.flush()?;
    Ok(())
}

/// Drop and recreate `ingest_staging.stg_tx_results_sos_civix` so each run starts with a fresh table.
/// Schema matches ingest_staging.stg_tx_results_sos so the same merge CTE can be used.
pub async fn ensure_stg_tx_results_sos_civix_table(
    pool: &PgPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;
    sqlx::query("DROP TABLE IF EXISTS ingest_staging.stg_tx_results_sos_civix")
        .execute(pool)
        .await?;
    sqlx::query(
        r#"
        CREATE TABLE ingest_staging.stg_tx_results_sos_civix (
            id BIGSERIAL PRIMARY KEY,
            office_name TEXT,
            office_key TEXT,
            candidate_name TEXT,
            candidate_key TEXT,
            precincts_reporting BIGINT,
            precincts_total BIGINT,
            votes_for_candidate BIGINT,
            total_votes BIGINT,
            total_voters BIGINT,
            party TEXT,
            race_type TEXT,
            election_year INTEGER,
            ref_key TEXT NOT NULL,
            source_file TEXT,
            ingested_at TIMESTAMPTZ NOT NULL DEFAULT (now() AT TIME ZONE 'utc')
        )
        "#,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Normalize party for staging: R -> REP, D -> DEM; otherwise return as-is.
fn party_for_staging(party: &str) -> String {
    match party.trim() {
        "R" => "REP".to_string(),
        "D" => "DEM".to_string(),
        other => other.to_string(),
    }
}

const CIVIX_SOURCE_FILE: &str = "SOS Civix website scrape";

/// Drop/recreate table and insert all rows into `ingest_staging.stg_tx_results_sos_civix`. Returns number of rows inserted.
/// Uses same column shape as stg_tx_results_sos; only set columns are written, rest stay NULL. ingested_at defaults to insert time.
/// Uses a single transaction to avoid a commit round-trip per row.
pub async fn write_results_to_db(
    pool: &PgPool,
    rows: &[ResultRow],
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    ensure_stg_tx_results_sos_civix_table(pool).await?;
    let mut tx = pool.begin().await?;
    let mut count = 0u64;
    for r in rows {
        let party = party_for_staging(&r.party);
        sqlx::query(
            r#"
            INSERT INTO ingest_staging.stg_tx_results_sos_civix (
                office_name, candidate_name, party, votes_for_candidate, total_votes, ref_key, source_file
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(&r.race)
        .bind(&r.choice)
        .bind(&party)
        .bind(r.votes_for_candidate as i64)
        .bind(r.total_votes as i64)
        .bind(&r.ref_key)
        .bind(CIVIX_SOURCE_FILE)
        .execute(&mut *tx)
        .await?;
        count += 1;
    }
    tx.commit().await?;
    Ok(count)
}
