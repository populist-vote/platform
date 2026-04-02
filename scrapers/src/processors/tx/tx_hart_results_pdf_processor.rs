//! Texas Hart/CIRA Cumulative Election Results PDF → CSV.
//!
//! Parses PDF text (proposition, general, primary), extracts race/choice/party and vote columns.
//! Output is PDF-style CSV: col_order as headers (e.g. Ballot by Mail, Early Voting,
//! Election Day Voting, Total) with votes and pct per cell, then summary columns (Cast Votes,
//! Undervotes, Overvotes) per voting method, plus precinct and county.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use csv::WriterBuilder;
use once_cell::sync::Lazy;
use pdf_extract::extract_text_from_mem;
use regex::Regex;

// ── Patterns (equivalent to Python) ───────────────────────────────────────────

/// One (votes, percent) pair for variable-column matching.
static VOTE_PAIR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"([\d,]+)\s*([\d.]+%?)").unwrap());

/// One numeric value per column (no percent). Used for Undervotes/Overvotes summary lines.
/// Optional decimal part (e.g. 10.0) so one number stays one value and indices don't shift.
static VOTE_VALUE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"([\d,]+(?:\.\d*)?)").unwrap());

/// Prefix for candidate-with-party lines: name and party, then vote tail.
static CANDIDATE_WITH_PARTY_PREFIX_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(.+?)\s+(REP|DEM|LIB|GRN|IND|NPA|\(W\))\s+(.+)$").unwrap()
});

/// Prefix for summary lines: "Cast Votes:" etc then vote tail.
static SUMMARY_PREFIX_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(Cast Votes:|Undervotes:|Overvotes:|Rejected write-in votes:|Unresolved write-in votes:)\s+(.+)$").unwrap()
});

/// Captures proposition choice (FOR, AGAINST, Yes, or No) and vote tail. \b ensures "Yes"/"No" are whole words (not "Yesterday"/"Nothing").
static PROPOSITION_CHOICE_PARSE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(FOR|AGAINST|Yes|No)\b\s+(.+)$").unwrap());

/// Detects proposition choice lines (FOR/AGAINST/Yes/No + vote data) so we don't treat them as race titles. \b for whole-word Yes/No.
static PROPOSITION_CHOICE_DETECT_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(FOR|AGAINST|Yes|No)\b\s+.+[\d,]+").unwrap());

/// Detect candidate-with-party line: has party code then a vote-like tail (at least one pair).
static CANDIDATE_WITH_PARTY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(.+?)\s+(REP|DEM|LIB|GRN|IND|NPA|\(W\))\s+.+[\d,]+\s+[\d.]+%").unwrap()
});

/// Detect candidate-without-party: line ending with vote-like tail (at least one pair).
static CANDIDATE_NO_PARTY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^.+\s+[\d,]+\s+[\d.]+%\s*$").unwrap()
});

/// Detect summary line.
static SUMMARY_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(Cast Votes:|Undervotes:|Overvotes:|Rejected write-in votes:|Unresolved write-in votes:)\s+").unwrap()
});

static HEADER_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^Choice\s+Party\s+(.+)").unwrap());

/// Document-wide: "X of Y = Z%"
static PRECINCTS_REPORTING_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\d+)\s+of\s+(\d+)\s*=\s*([\d.]+%)").unwrap());

/// Per-race (Matagorda style): "23 23 100.00% 13,359 22,338 59.80%"
static PER_RACE_PRECINCTS_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(\d+)\s+(\d+)\s+([\d.]+%)\s+([\d,]+)\s+([\d,]+)\s+([\d.]+%)").unwrap()
});

/// Lines that are not race titles (boilerplate, metadata, headers). Skip these when detecting race/contest names.
static NON_RACE_TITLE_LINE_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"^(Choice\s|Cumulative Results|Run Time|Run Date|Registered Voters|Precincts Reporting|Precincts Counted|Unofficial|Official|Page \d+|\d+ of \d+|\d+/\d+/\d+|\*\*\*|PRIMARY ELECTION|GENERAL ELECTION|CONSTITUTIONAL AMENDMENT|NOVEMBER|MARCH|JOINT PRIMARY|Counted\s+Total\s+Percent|Voters|Ballots\s+Registered)",
    )
    .unwrap()
});

/// Line that ends with number and percent (trailing vote-like); used to reject race titles.
static ENDS_NUM_PCT_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\d+\s+[\d.]+%\s*$").unwrap());

// ── Internal parsed row (CIRA format) ─────────────────────────────────────────

#[derive(Debug, Clone)]
struct ParsedRow {
    race: String,
    choice: String,
    party: String,
    total_votes: String,
    total_pct: String,
    /// Per-column (votes, pct) pairs in col_order order, for PDF-style CSV output.
    column_pairs: Vec<(String, String)>,
    precincts_counted: String,
    precincts_total: String,
    precincts_pct: String,
}

/// Result of parsing a Hart PDF: column order, all rows with per-column pairs, and county.
#[derive(Debug, Clone)]
pub struct HartPdfResult {
    pub col_order: Vec<String>,
    rows: Vec<ParsedRow>,
    pub county: String,
}

impl HartPdfResult {
    /// Number of parsed rows (choices) in this result.
    pub fn row_count(&self) -> usize {
        self.rows.len()
    }
}

/// Extract all (votes, pct) pairs from a line tail in left-to-right order.
fn parse_vote_pairs(tail: &str) -> Vec<(String, String)> {
    VOTE_PAIR_RE
        .captures_iter(tail)
        .filter_map(|cap| {
            let votes = cap.get(1)?.as_str().to_string();
            let pct = cap.get(2)?.as_str().to_string();
            Some((votes, pct))
        })
        .collect()
}

/// Extract numeric values only (one per column), for summary lines that have no pct (e.g. Undervotes, Overvotes).
fn parse_vote_values_single(tail: &str) -> Vec<String> {
    VOTE_VALUE_RE
        .find_iter(tail)
        .map(|m| m.as_str().to_string())
        .collect()
}

/// Column header phrases (case-insensitive) that map to our canonical CSV column names.
/// (phrase to find in header, canonical CSV column name). Matching is case-insensitive; phrases are lowercase here for consistency.
/// "election day" matches both "Election Day" and "Election Day Voting".
const COLUMN_PHRASES: &[(&str, &str)] = &[
    ("ballot by mail", "Ballot by Mail"),
    ("absentee voting", "Ballot by Mail"),
    ("election day", "Election Day Voting"),
    ("early voting", "Early Voting"),
    ("absentee", "Ballot by Mail"),
    ("total", "Total"),
];

/// Phrases that identify columns we skip (e.g. provisional). Each phrase can match multiple columns;
/// we add one skip entry per occurrence so "provisional" skips "EV Provisional", "ED Provisional", and standalone "Provisional".
const COLUMN_SKIP_PHRASES: &[&str] = &["limited", "provisional"];

/// Parsed column order: canonical names and their 0-based PDF column indices (so we can skip provisional).
fn parse_column_order(header_rest: &str) -> (Vec<String>, Vec<usize>) {
    let header_lower = header_rest.to_lowercase();

    // For each canonical, take leftmost phrase position (dedupe e.g. "Election day" vs "Election Day Voting").
    let mut by_canonical: HashMap<String, usize> = HashMap::new();
    for (phrase, canonical) in COLUMN_PHRASES {
        if let Some(pos) = header_lower.find(&phrase.to_lowercase()) {
            by_canonical
                .entry(canonical.to_string())
                .and_modify(|p| {
                    if pos < *p {
                        *p = pos;
                    }
                })
                .or_insert(pos);
        }
    }

    // Collect (position, optional canonical). None = skip column (provisional).
    let mut entries: Vec<(usize, Option<String>)> = by_canonical
        .into_iter()
        .map(|(c, pos)| (pos, Some(c)))
        .collect();
    for phrase in COLUMN_SKIP_PHRASES {
        let phrase_lower = phrase.to_lowercase();
        let mut search_from = 0;
        while let Some(rel_pos) = header_lower[search_from..].find(&phrase_lower) {
            let pos = search_from + rel_pos;
            entries.push((pos, None));
            search_from = pos + 1;
        }
    }

    // Sort by position; PDF column index is 0, 1, 2, ... by order in header.
    entries.sort_by_key(|(pos, _)| *pos);
    let mut col_order: Vec<String> = Vec::new();
    let mut col_indices: Vec<usize> = Vec::new();
    for (pdf_index, (_, canonical)) in entries.into_iter().enumerate() {
        if let Some(c) = canonical {
            col_order.push(c);
            col_indices.push(pdf_index);
        }
    }
    (col_order, col_indices)
}

fn looks_like_race(line: &str) -> bool {
    if line.is_empty() || NON_RACE_TITLE_LINE_RE.is_match(line) {
        return false;
    }
    if PROPOSITION_CHOICE_DETECT_RE.is_match(line) || SUMMARY_RE.is_match(line) {
        return false;
    }
    if CANDIDATE_WITH_PARTY_RE.is_match(line) || PER_RACE_PRECINCTS_RE.is_match(line) {
        return false;
    }
    if line.starts_with(|c: char| c.is_ascii_digit()) {
        return false;
    }
    if ENDS_NUM_PCT_RE.is_match(line) {
        return false;
    }
    true
}

/// Remove space between single letter and following lowercase (e.g. "Mc Donald" -> "McDonald").
fn clean_name(name: &str) -> String {
    let re = Regex::new(r"([A-Za-z]) ([a-z])").unwrap();
    re.replace_all(name, "$1$2").trim().to_string()
}

fn make_row(
    race: &str,
    col_order: &[String],
    col_indices: &[usize],
    choice: &str,
    party: &str,
    pairs: &[(String, String)],
    precincts_counted: &str,
    precincts_total: &str,
    precincts_pct: &str,
) -> ParsedRow {
    // Map by PDF column index: i-th output column uses pairs[col_indices[i]] (so we skip provisional).
    let column_pairs: Vec<(String, String)> = col_order
        .iter()
        .enumerate()
        .map(|(i, _col)| {
            let idx = col_indices.get(i).copied().unwrap_or(i);
            pairs
                .get(idx)
                .map(|(v, p)| (v.replace(',', ""), p.clone()))
                .unwrap_or_else(|| (String::new(), String::new()))
        })
        .collect();
    let (tot_v, tot_p) = col_order
        .iter()
        .position(|c| c.as_str() == "Total")
        .and_then(|i| column_pairs.get(i))
        .map(|(v, p)| (v.clone(), p.clone()))
        .unwrap_or_else(|| (String::new(), String::new()));
    let party = if party.is_empty() {
        let race_lower = race.to_lowercase();
        if race_lower.contains("republican party") {
            "REP"
        } else if race_lower.contains("democratic party") {
            "DEM"
        } else {
            ""
        }
    } else {
        party
    };
    ParsedRow {
        race: race.to_string(),
        choice: choice.to_string(),
        party: party.to_string(),
        total_votes: tot_v,
        total_pct: tot_p,
        column_pairs,
        precincts_counted: precincts_counted.to_string(),
        precincts_total: precincts_total.to_string(),
        precincts_pct: precincts_pct.to_string(),
    }
}

/// Extract county name from first page text (e.g. "MATAGORDA COUNTY, TEXAS" -> "Matagorda").
fn detect_county_from_text(text: &str) -> Option<String> {
    let county_texas_re = Regex::new(r"(?i)COUNTY,\s+TEXAS").ok()?;
    for line in text.lines().take(10) {
        if let Some(m) = county_texas_re.find(line) {
            let before = line[..m.start()].trim();
            let words: Vec<&str> = before.split_whitespace().collect();
            if words.is_empty() {
                continue;
            }
            let multi_prefix = ["SAN", "EL", "DE", "LA", "VAN"];
            let county = if words.len() >= 2
                && multi_prefix
                    .iter()
                    .any(|p| p.eq_ignore_ascii_case(words[words.len() - 2]))
            {
                format!("{} {}", words[words.len() - 2], words[words.len() - 1])
            } else {
                words[words.len() - 1].to_string()
            };
            return Some(
                county
                    .chars()
                    .enumerate()
                    .map(|(i, c)| {
                        if i == 0 {
                            c.to_uppercase().to_string()
                        } else {
                            c.to_lowercase().to_string()
                        }
                    })
                    .collect::<String>(),
            );
        }
    }
    None
}

/// Parse Hart/CIRA PDF; returns column order, parsed rows (with per-column pairs), and county.
pub fn parse_hart_pdf(
    pdf_path: &Path,
    county_name: &str,
) -> Result<HartPdfResult, Box<dyn std::error::Error + Send + Sync>> {
    let bytes = fs::read(pdf_path)?;
    let text = extract_text_from_mem(&bytes)
        .map_err(|e| format!("PDF text extraction failed: {}", e))?;

    let detected_county = detect_county_from_text(&text);
    println!(
        "County: scanned from PDF = {}, passed in = '{}'",
        detected_county
            .as_deref()
            .unwrap_or("(none)"),
        county_name
    );

    let mut rows: Vec<ParsedRow> = Vec::new();
    let mut current_race: Option<String> = None;
    let default_order = vec![
        "Ballot by Mail".into(),
        "Early Voting".into(),
        "Election Day Voting".into(),
        "Total".into(),
    ];
    let default_indices = vec![0, 1, 2, 3];
    let mut col_order: Vec<String> = default_order.clone();
    let mut col_indices: Vec<usize> = default_indices.clone();

    let mut doc_precincts_counted = String::new();
    let mut doc_precincts_total = String::new();
    let mut doc_precincts_pct = String::new();
    // Pre-scan: only use "X of Y = Z%" when it appears after "Precincts Reporting", so we
    // don't capture Registered Voters or other numeric lines (e.g. Hays PDF).
    let precincts_reporting_label = Regex::new(r"^Precincts Reporting\s*$").unwrap();
    let registered_voters_label = Regex::new(r"^Registered Voters\s*$").unwrap();
    let mut saw_precincts_in_prescan = false;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        if registered_voters_label.is_match(line) {
            saw_precincts_in_prescan = false;
            continue;
        }
        if precincts_reporting_label.is_match(line) {
            saw_precincts_in_prescan = true;
            continue;
        }
        if saw_precincts_in_prescan {
            if let Some(caps) = PRECINCTS_REPORTING_RE.captures(line) {
                doc_precincts_counted = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                doc_precincts_total = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
                doc_precincts_pct = caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default();
                break;
            }
            if line.split_whitespace().count() > 3 || line.starts_with(|c: char| c.is_ascii_digit()) {
                saw_precincts_in_prescan = false;
            }
        }
    }

    let mut race_precincts_counted = String::new();
    let mut race_precincts_total = String::new();
    let mut race_precincts_pct = String::new();
    let mut saw_precincts_reporting_label = false;
    let mut saw_per_race_header = false;

    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        // Don't use "X of Y = Z%" that follows Registered Voters (we never use Registered Voters).
        if Regex::new(r"^Registered Voters\s*$").unwrap().is_match(line) {
            saw_precincts_reporting_label = false;
            continue;
        }

        // Detect "Precincts Reporting" label (doc-wide)
        if Regex::new(r"^Precincts Reporting\s*$")
            .unwrap()
            .is_match(line)
        {
            saw_precincts_reporting_label = true;
            continue;
        }

        // Capture doc-wide "X of Y = Z%"
        if saw_precincts_reporting_label {
            if let Some(caps) = PRECINCTS_REPORTING_RE.captures(line) {
                saw_precincts_reporting_label = false;
                doc_precincts_counted = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                doc_precincts_total = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
                doc_precincts_pct = caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default();
                race_precincts_counted.clear();
                race_precincts_total.clear();
                race_precincts_pct.clear();
                continue;
            }
            if line.split_whitespace().count() <= 3 && !line.starts_with(|c: char| c.is_ascii_digit())
            {
                continue;
            }
            saw_precincts_reporting_label = false;
        }

        // Per-race precincts header (Matagorda style)
        if Regex::new(r"^Precincts\s+Voters\s*$").unwrap().is_match(line) {
            saw_per_race_header = true;
            continue;
        }
        if saw_per_race_header {
            if Regex::new(r"^Counted\s+Total\s+Percent").unwrap().is_match(line) {
                continue;
            }
            saw_per_race_header = false;
            if let Some(caps) = PER_RACE_PRECINCTS_RE.captures(line) {
                race_precincts_counted = caps.get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
                race_precincts_total = caps.get(2).map(|m| m.as_str().to_string()).unwrap_or_default();
                race_precincts_pct = caps.get(3).map(|m| m.as_str().to_string()).unwrap_or_default();
                continue;
            }
        }

        // Column header
        if let Some(caps) = HEADER_RE.captures(line) {
            let rest = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let (order, indices) = parse_column_order(rest);
            if !order.is_empty() {
                col_order = order;
                col_indices = indices;
            }
            continue;
        }

        // Race title
        if looks_like_race(line) {
            current_race = Some(line.to_string());
            race_precincts_counted.clear();
            race_precincts_total.clear();
            race_precincts_pct.clear();
            continue;
        }

        let race = match &current_race {
            Some(r) => r.as_str(),
            None => continue,
        };

        let p_counted = if race_precincts_counted.is_empty() {
            doc_precincts_counted.as_str()
        } else {
            race_precincts_counted.as_str()
        };
        let p_total = if race_precincts_total.is_empty() {
            doc_precincts_total.as_str()
        } else {
            race_precincts_total.as_str()
        };
        let p_pct = if race_precincts_pct.is_empty() {
            doc_precincts_pct.as_str()
        } else {
            race_precincts_pct.as_str()
        };

        // Proposition choice (FOR / AGAINST / Yes / No)
        if let Some(caps) = PROPOSITION_CHOICE_PARSE_RE.captures(line) {
            let choice = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let tail = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let pairs = parse_vote_pairs(tail);
            rows.push(make_row(
                race,
                &col_order,
                &col_indices,
                choice,
                "",
                &pairs,
                p_counted,
                p_total,
                p_pct,
            ));
            continue;
        }

        // Summary rows (Cast Votes:, Undervotes:, etc.)
        if let Some(caps) = SUMMARY_PREFIX_RE.captures(line) {
            let choice = caps
                .get(1)
                .map(|m| m.as_str().trim_end_matches(':'))
                .unwrap_or("");
            let tail = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            if choice == "Undervotes" || choice == "Overvotes" {
                // Single value per column (no vote/pct pairs). Some PDFs omit provisional columns on this row,
                // so we may get fewer values than PDF columns. If value count matches col_order, use 1:1 mapping;
                // otherwise use col_indices to pick from full PDF column list.
                let values = parse_vote_values_single(tail);
                let pairs: Vec<(String, String)> = values
                    .into_iter()
                    .map(|v| (v, String::new()))
                    .collect();
                if pairs.len() == col_order.len() {
                    // Row has one value per output column in order (no provisional on this row).
                    let one_to_one: Vec<usize> = (0..col_order.len()).collect();
                    rows.push(make_row(
                        race,
                        &col_order,
                        &one_to_one,
                        choice,
                        "",
                        &pairs,
                        p_counted,
                        p_total,
                        p_pct,
                    ));
                } else {
                    rows.push(make_row(
                        race,
                        &col_order,
                        &col_indices,
                        choice,
                        "",
                        &pairs,
                        p_counted,
                        p_total,
                        p_pct,
                    ));
                }
            } else {
                let pairs = parse_vote_pairs(tail);
                rows.push(make_row(
                    race,
                    &col_order,
                    &col_indices,
                    choice,
                    "",
                    &pairs,
                    p_counted,
                    p_total,
                    p_pct,
                ));
            }
            continue;
        }

        // Candidate with party
        if let Some(caps) = CANDIDATE_WITH_PARTY_PREFIX_RE.captures(line) {
            let name = clean_name(caps.get(1).map(|m| m.as_str()).unwrap_or(""));
            let party = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let tail = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            let pairs = parse_vote_pairs(tail);
            rows.push(make_row(
                race,
                &col_order,
                &col_indices,
                &name,
                party,
                &pairs,
                p_counted,
                p_total,
                p_pct,
            ));
            continue;
        }

        // Candidate without party: everything before the first vote pair is the name.
        if CANDIDATE_NO_PARTY_RE.is_match(line) {
            if let Some(m) = VOTE_PAIR_RE.find(line) {
                let name = clean_name(line[..m.start()].trim());
                let tail = &line[m.start()..];
                let pairs = parse_vote_pairs(tail);
                rows.push(make_row(
                    race,
                    &col_order,
                    &col_indices,
                    &name,
                    "",
                    &pairs,
                    p_counted,
                    p_total,
                    p_pct,
                ));
                continue;
            }
        }
    }

    let county = if county_name.is_empty() {
        detected_county.unwrap_or_default()
    } else {
        county_name.to_string()
    };

    Ok(HartPdfResult {
        col_order,
        rows,
        county,
    })
}

/// Summary choice names that get one column per voting-method in the CSV (votes only, by column index).
const SUMMARY_VOTE_COLUMN_SUFFIXES: &[&str] = &["Cast Votes", "Undervotes", "Overvotes"];

/// Write PDF-style CSV: for each col_order column, two columns (votes, pct) then "[Col] Cast Votes",
/// "[Col] Undervotes", "[Col] Overvotes" (summary vote value for that column only). Then Precincts, County.
pub fn write_pdf_style_csv(
    result: &HartPdfResult,
    csv_path: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut race_summary_columns: HashMap<String, HashMap<String, Vec<(String, String)>>> =
        HashMap::new();
    for row in &result.rows {
        if SUMMARY_VOTE_COLUMN_SUFFIXES.contains(&row.choice.as_str()) {
            race_summary_columns
                .entry(row.race.clone())
                .or_default()
                .insert(row.choice.clone(), row.column_pairs.clone());
        }
    }

    let mut headers: Vec<String> = vec!["Race".into(), "Choice".into(), "Party".into()];
    for col in &result.col_order {
        headers.push(col.clone());
        headers.push(format!("{} Pct", col));
        for summary_name in SUMMARY_VOTE_COLUMN_SUFFIXES {
            headers.push(format!("{} {}", col, summary_name));
        }
    }
    headers.push("Precincts Counted".into());
    headers.push("Precincts Total".into());
    headers.push("Precincts Pct".into());
    headers.push("County".into());

    let mut wtr = WriterBuilder::new().from_path(csv_path)?;
    wtr.write_record(&headers)?;

    for row in &result.rows {
        if SUMMARY_VOTE_COLUMN_SUFFIXES.contains(&row.choice.as_str()) {
            continue;
        }
        let mut record: Vec<String> = vec![
            row.race.clone(),
            row.choice.clone(),
            row.party.clone(),
        ];
        let race_summaries = race_summary_columns.get(&row.race);
        for (i, (votes, pct)) in row.column_pairs.iter().enumerate() {
            record.push(votes.clone());
            record.push(pct.clone());
            for summary_name in SUMMARY_VOTE_COLUMN_SUFFIXES {
                let val = race_summaries
                    .and_then(|m| m.get(*summary_name))
                    .and_then(|pairs| pairs.get(i))
                    .map(|(v, _p)| v.clone())
                    .unwrap_or_else(|| ", ".to_string());
                record.push(val);
            }
        }
        record.push(row.precincts_counted.clone());
        record.push(row.precincts_total.clone());
        record.push(row.precincts_pct.clone());
        record.push(result.county.clone());
        wtr.write_record(&record)?;
    }
    wtr.flush()?;
    Ok(())
}

/// Parse Hart PDF and write PDF-style CSV. If `county_name` is empty, attempt to detect from PDF.
pub fn parse_hart_pdf_to_csv(
    pdf_path: &Path,
    csv_path: Option<&Path>,
    county_name: &str,
) -> Result<HartPdfResult, Box<dyn std::error::Error + Send + Sync>> {
    let result = parse_hart_pdf(pdf_path, county_name)?;
    let csv_path_final = match csv_path {
        Some(p) => p.to_path_buf(),
        None => pdf_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(pdf_path.file_stem().unwrap_or(std::ffi::OsStr::new("output")))
            .with_extension("csv"),
    };
    write_pdf_style_csv(&result, &csv_path_final)?;
    Ok(result)
}
