//! Unified TX election results processor: SOS (XML), Clarity, Hart, and Other county CSVs.
//!
//! Shared county-level office matching (COUNTY_OFFICE_MATCH_RULES) and ref_key building
//! (build_ref_key_for_county_race) for Clarity, Hart, and Other. SOS uses a separate XML path
//! and PoliticianRefKeyGenerator.

use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Mutex;

use csv::{ReaderBuilder, StringRecord};
use once_cell::sync::Lazy;
use regex::Regex;
use roxmltree::Document;
use sqlx::PgPool;
use slugify::slugify;

use crate::extractors::politician;
use crate::extractors::tx::tx_office::{extract_office_district, extract_office_seat};
use crate::generators::politician::PoliticianRefKeyGenerator;
use crate::util::decode_csv_bytes_to_utf8;

const DEFAULT_ELECTION_YEAR: i32 = 2026;

// ---------- SOS (XML) ----------

/// Default directory for TX SOS XML files (relative to scrapers crate root).
const TX_SOS_DATA_DIR: &str = "data/tx/sos";

/// One row written to ingest_staging.stg_tx_results_sos.
#[derive(Debug, Clone)]
pub struct StgTxResultRow {
    pub office_name: Option<String>,
    pub office_key: Option<String>,
    pub candidate_name: Option<String>,
    pub candidate_key: Option<String>,
    pub precincts_reporting: Option<i64>,
    pub precincts_total: Option<i64>,
    pub votes_for_candidate: Option<i64>,
    pub total_votes: Option<i64>,
    pub total_voters: Option<i64>,
    pub party: Option<String>,
    pub race_type: Option<String>,
    pub election_year: Option<i32>,
    pub ref_key: String,
    pub source_file: Option<String>,
}

pub async fn ensure_staging_table(pool: &PgPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query("CREATE SCHEMA IF NOT EXISTS ingest_staging")
        .execute(pool)
        .await?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS ingest_staging.stg_tx_results_sos (
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

fn tx_sos_data_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    Path::new(&manifest_dir).join(TX_SOS_DATA_DIR)
}

pub fn list_tx_sos_xml_files() -> Result<Vec<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    let dir = tx_sos_data_path();
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut files: Vec<_> = fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |e| e == "xml"))
        .collect();
    files.sort();
    Ok(files)
}

fn attr_parse_u64(node: &roxmltree::Node, name: &str) -> Option<u64> {
    node.attribute(name).and_then(|s| u64::from_str(s.trim()).ok())
}

fn normalize_candidate_name(raw: &str) -> Option<String> {
    let without_incumbent = raw.trim().replace("(I)", "").trim().to_string();
    Some(politician::strip_accents(&without_incumbent).trim().to_string())
}

fn parse_party_from_election_name(name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    if lower.contains("republican") {
        Some("republican".to_string())
    } else if lower.contains("democratic") {
        Some("democratic".to_string())
    } else {
        None
    }
}

fn parse_race_type_from_election_name(name: &str) -> Option<String> {
    if name.to_lowercase().contains("primary") {
        Some("primary".to_string())
    } else {
        None
    }
}

fn parse_year_from_election_date(date: Option<&str>) -> Option<i32> {
    let s = date?.trim();
    let year_str = s.get(0..4)?;
    year_str.parse().ok()
}

pub fn parse_tx_sos_xml(content: &str, source_file: &str) -> Result<Vec<StgTxResultRow>, Box<dyn std::error::Error + Send + Sync>> {
    let doc = Document::parse(content)?;
    let root = doc.root_element();
    let mut rows = Vec::new();
    for election_result in root.children().filter(|n| n.has_tag_name("ElectionResult")) {
        let election_name = election_result.attribute("ElectionName").unwrap_or("");
        let election_date = election_result.attribute("ElectionDate");
        let party = parse_party_from_election_name(election_name);
        let race_type = parse_race_type_from_election_name(election_name);
        let election_year = parse_year_from_election_date(election_date);
        for race in election_result.children().filter(|n| n.has_tag_name("Race")) {
            let office_name = race.attribute("name").map(|s| s.trim().to_string());
            let office_key = race.attribute("key").map(String::from);
            let precincts_reporting = race
                .attribute("precinctsReported")
                .and_then(|s| u64::from_str(s.trim()).ok())
                .map(|u| u as i64);
            let precincts_total = race
                .attribute("precinctsParticipating")
                .and_then(|s| u64::from_str(s.trim()).ok())
                .map(|u| u as i64);
            let candidates: Vec<_> = race
                .children()
                .filter(|n| n.has_tag_name("Candidate"))
                .collect();
            if candidates.is_empty() {
                continue;
            }
            let candidate_votes: Vec<i64> = candidates
                .iter()
                .map(|c| {
                    let early = attr_parse_u64(c, "earlyBallotsCast").unwrap_or(0);
                    let day = attr_parse_u64(c, "ballotsCast").unwrap_or(0);
                    (early + day) as i64
                })
                .collect();
            let total_votes_race: i64 = candidate_votes.iter().sum();
            for (c, votes_for_candidate) in candidates.iter().zip(candidate_votes.iter().copied()) {
                let candidate_name = c.attribute("name").and_then(normalize_candidate_name);
                let candidate_key = c.attribute("key").map(String::from);
                let total_voters = attr_parse_u64(c, "totalVoters").map(|u| u as i64);
                let ref_key = PoliticianRefKeyGenerator::new(
                    "tx-primaries",
                    election_year.unwrap_or(0),
                    office_name.as_deref().unwrap_or(""),
                    candidate_name.as_deref(),
                )
                .generate();
                rows.push(StgTxResultRow {
                    office_name: office_name.clone(),
                    office_key: office_key.clone(),
                    candidate_name,
                    candidate_key,
                    precincts_reporting,
                    precincts_total,
                    votes_for_candidate: Some(votes_for_candidate),
                    total_votes: Some(total_votes_race),
                    total_voters,
                    party: party.clone(),
                    race_type: race_type.clone(),
                    election_year,
                    ref_key,
                    source_file: Some(source_file.to_string()),
                });
            }
        }
    }
    Ok(rows)
}

pub async fn insert_sos_staging_rows(
    pool: &PgPool,
    rows: &[StgTxResultRow],
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    if rows.is_empty() {
        return Ok(0);
    }
    let mut count = 0u64;
    for row in rows {
        sqlx::query(
            r#"
            INSERT INTO ingest_staging.stg_tx_results_sos (
                office_name, office_key, candidate_name, candidate_key,
                precincts_reporting, precincts_total, votes_for_candidate, total_votes, total_voters,
                party, race_type, election_year, ref_key, source_file
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(&row.office_name)
        .bind(&row.office_key)
        .bind(&row.candidate_name)
        .bind(&row.candidate_key)
        .bind(row.precincts_reporting)
        .bind(row.precincts_total)
        .bind(row.votes_for_candidate)
        .bind(row.total_votes)
        .bind(row.total_voters)
        .bind(&row.party)
        .bind(&row.race_type)
        .bind(row.election_year)
        .bind(&row.ref_key)
        .bind(&row.source_file)
        .execute(pool)
        .await?;
        count += 1;
    }
    Ok(count)
}

pub async fn process_tx_sos_results(
    pool: &PgPool,
    clear_before_load: bool,
) -> Result<(usize, u64), Box<dyn std::error::Error + Send + Sync>> {
    ensure_staging_table(pool).await?;
    if clear_before_load {
        sqlx::query("TRUNCATE ingest_staging.stg_tx_results_sos")
            .execute(pool)
            .await?;
    }
    let files = list_tx_sos_xml_files()?;
    if files.is_empty() {
        return Ok((0, 0));
    }
    let mut files_processed = 0usize;
    let mut total_rows: u64 = 0;
    for path in &files {
        let content = fs::read_to_string(path)?;
        let source_file = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let rows = parse_tx_sos_xml(&content, &source_file)?;
        let n = insert_sos_staging_rows(pool, &rows).await?;
        files_processed += 1;
        total_rows += n;
        println!("  {}: {} rows", source_file, n);
    }
    Ok((files_processed, total_rows))
}

// ---------- Shared county rules (Clarity, Hart, Other) ----------

/// County office match rule: populist name, match/exclude substrings, optional regex.
struct CountyOfficeRule {
    populist_office_name: &'static str,
    match_substrings: &'static [&'static str],
    exclude_substrings: &'static [&'static str],
    match_pattern: Option<&'static str>,
}

/// Combined county-level office matching rules from Clarity, Hart, and Other.
/// Order matters: first match wins. More specific rules first.
const COUNTY_OFFICE_MATCH_RULES: &[CountyOfficeRule] = &[
    CountyOfficeRule {
        populist_office_name: "Judge - County Criminal Court of Appeals",
        match_substrings: &["Judge, County Criminal Court of Appeals"],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "Judge - County Criminal Court at Law",
        match_substrings: &["Judge, County Criminal Court"],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "Judge - County Court at Law",
        match_substrings: &["Judge, County Court at Law", "County Court-at-Law"],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "Judge - Probate Court",
        match_substrings: &["Probate Court"],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "Criminal District Judge",
        match_substrings: &["Criminal District Judge"],
        exclude_substrings: &["Judicial District"],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "Criminal District Attorney",
        match_substrings: &["Criminal District Attorney"],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "County Judge",
        match_substrings: &["County Judge"],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "Judge - County Civil Court at Law",
        match_substrings: &["Judge, County Civil Court at Law"],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "County Attorney",
        match_substrings: &["County Attorney"],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "County & District Clerk",
        match_substrings: &[
            "County and District Clerk",
            "District and County Clerk",
            "County Clerk/District Clerk",
            "District Clerk/County Clerk",
        ],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "District Clerk",
        match_substrings: &["District Clerk"],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "County Clerk",
        match_substrings: &["County Clerk"],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "Sheriff & County Tax Assessor-Collector",
        match_substrings: &["Sheriff and Tax Assessor-Collector"],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "Sheriff",
        match_substrings: &["Sheriff"],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "County Tax Assessor-Collector",
        match_substrings: &["Tax Assessor-Collector"],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "County Treasurer",
        match_substrings: &["County Treasurer"],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "County Surveyor",
        match_substrings: &["County Surveyor"],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "County School Trustee",
        match_substrings: &["County School Trustee"],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "County Commissioner",
        match_substrings: &["County Commissioner", "Commissioner"],
        exclude_substrings: &["Railroad", "Agriculture", "General Land Office"],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "Justice of the Peace",
        match_substrings: &["Justice of the Peace", "JOP"],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "County Constable",
        match_substrings: &["Constable"],
        exclude_substrings: &[],
        match_pattern: None,
    },
    CountyOfficeRule {
        populist_office_name: "Precinct Chair",
        match_substrings: &["Precinct Chair"],
        exclude_substrings: &[],
        match_pattern: Some(r"(?i)precinct\s+[0-9]+\s+chair"),
    },
    CountyOfficeRule {
        populist_office_name: "County Chair",
        match_substrings: &["County Chair", "Party Chairman", "Party Chair"],
        exclude_substrings: &[],
        match_pattern: None,
    },
];

static RULE_PATTERN_CACHE: Lazy<Mutex<HashMap<&'static str, Regex>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

fn rule_pattern_matches(contest_lower: &str, pattern: &'static str) -> bool {
    let mut cache = RULE_PATTERN_CACHE.lock().unwrap();
    let re = cache
        .entry(pattern)
        .or_insert_with(|| Regex::new(pattern).unwrap());
    re.is_match(contest_lower)
}

fn find_matching_county_office(normalized_contest_name: &str) -> Option<&'static CountyOfficeRule> {
    let contest_lower = normalized_contest_name.to_lowercase();
    for rule in COUNTY_OFFICE_MATCH_RULES {
        if rule.match_substrings.is_empty() && rule.match_pattern.is_none() {
            continue;
        }
        let matched_substring = rule
            .match_substrings
            .iter()
            .any(|s| contest_lower.contains(&s.to_lowercase()));
        let matched_pattern = rule
            .match_pattern
            .map_or(false, |pat| rule_pattern_matches(&contest_lower, pat));
        if !matched_substring && !matched_pattern {
            continue;
        }
        let excluded = rule
            .exclude_substrings
            .iter()
            .any(|s| contest_lower.contains(&s.to_lowercase()));
        if excluded {
            continue;
        }
        return Some(rule);
    }
    None
}

fn remove_substring_ignore_ascii_case(s: &str, phrase: &str) -> String {
    if phrase.is_empty() {
        return s.to_string();
    }
    let lower = s.to_lowercase();
    let phrase_lower = phrase.to_lowercase();
    let mut result = String::new();
    let mut search_start = 0;
    while let Some(pos) = lower[search_start..].find(&phrase_lower) {
        let pos = search_start + pos;
        result.push_str(&s[search_start..pos]);
        search_start = pos + phrase.len();
    }
    result.push_str(&s[search_start..]);
    result
}

fn normalize_contest_name(s: &str) -> String {
    let mut s: String = remove_substring_ignore_ascii_case(s.trim(), "unexpired term")
        .trim()
        .trim_end_matches(|c| c == ' ' || c == '-')
        .into();
    let suffix1 = " - Vote for none or one";
    if s.len() >= suffix1.len() && s[s.len() - suffix1.len()..].eq_ignore_ascii_case(suffix1) {
        s = s[..s.len() - suffix1.len()].trim_end().to_string();
    }
    let suffix_rep = " - Republican Party";
    let suffix_dem = " - Democratic Party";
    if s.len() >= suffix_rep.len() && s[s.len() - suffix_rep.len()..].eq_ignore_ascii_case(suffix_rep) {
        s = s[..s.len() - suffix_rep.len()].trim_end().to_string();
    } else if s.len() >= suffix_dem.len() && s[s.len() - suffix_dem.len()..].eq_ignore_ascii_case(suffix_dem) {
        s = s[..s.len() - suffix_dem.len()].trim_end().to_string();
    }
    let suffix_d = " (D)";
    let suffix_r = " (R)";
    if s.len() >= suffix_d.len() && s[s.len() - suffix_d.len()..].eq_ignore_ascii_case(suffix_d) {
        s = s[..s.len() - suffix_d.len()].trim_end().to_string();
    } else if s.len() >= suffix_r.len() && s[s.len() - suffix_r.len()..].eq_ignore_ascii_case(suffix_r) {
        s = s[..s.len() - suffix_r.len()].trim_end().to_string();
    }
    let prefix_rep = "REP - ";
    let prefix_dem = "DEM - ";
    if s.len() >= prefix_rep.len() && s[..prefix_rep.len()].eq_ignore_ascii_case(prefix_rep) {
        s = s[prefix_rep.len()..].trim_start().to_string();
    } else if s.len() >= prefix_dem.len() && s[..prefix_dem.len()].eq_ignore_ascii_case(prefix_dem) {
        s = s[prefix_dem.len()..].trim_start().to_string();
    }
    let suffix_vote1 = "(Vote for 1)";
    if s.len() >= suffix_vote1.len() && s[s.len() - suffix_vote1.len()..].eq_ignore_ascii_case(suffix_vote1) {
        s = s[..s.len() - suffix_vote1.len()].trim_end().to_string();
    }
    s
}

#[derive(Debug, Clone)]
struct ParsedContestName {
    office_name: String,
    district: Option<String>,
    seat: Option<String>,
}

fn parse_contest_name_for_office(rule: &CountyOfficeRule, contest_name: &str) -> ParsedContestName {
    let (seat, stripped) = extract_office_seat(contest_name);
    let district = extract_office_district(&stripped, None);
    ParsedContestName {
        office_name: rule.populist_office_name.to_string(),
        district,
        seat,
    }
}

fn push_slug(parts: &mut Vec<String>, s: &str) {
    let t = s.trim();
    if !t.is_empty() {
        let slug = slugify!(t);
        if !slug.is_empty() {
            parts.push(slug);
        }
    }
}

/// Build ref_key for a county race row. Format varies by office and county.
fn build_ref_key_for_county_race(
    office_name: &str,
    parsed: &ParsedContestName,
    county: Option<&str>,
    candidate_name: Option<&str>,
    party: Option<&str>,
    year: i32,
) -> String {
    let county = county.unwrap_or("").trim();
    let county_lower = county.to_lowercase();
    let candidate = candidate_name.unwrap_or("").trim();
    let district = parsed
        .district
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());
    let seat = parsed
        .seat
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty());
    let party = party.map(|s| s.trim()).filter(|s| !s.is_empty());
    let mut parts: Vec<String> = vec!["tx-primaries".to_string(), year.to_string()];
    match office_name {
        "Criminal District Judge" => {
            if county_lower == "tarrant" {
                push_slug(&mut parts, office_name);
                if let Some(d) = district {
                    push_slug(&mut parts, d);
                }
                push_slug(&mut parts, &format!("{} county", county));
                push_slug(&mut parts, candidate);
            } else if county_lower == "dallas" {
                push_slug(&mut parts, office_name);
                push_slug(&mut parts, &format!("{} county number", county));
                if let Some(d) = district {
                    push_slug(&mut parts, d);
                }
                push_slug(&mut parts, candidate);
            } else {
                push_slug(&mut parts, office_name);
                push_slug(&mut parts, &format!("{} county", county));
                if let Some(d) = district {
                    push_slug(&mut parts, d);
                }
                push_slug(&mut parts, candidate);
            }
        }
        "Criminal District Attorney" => {
            push_slug(&mut parts, office_name);
            push_slug(&mut parts, &format!("{} county", county));
            push_slug(&mut parts, candidate);
        }
        "County Judge" => {
            push_slug(&mut parts, county);
            push_slug(&mut parts, office_name);
            push_slug(&mut parts, candidate);
        }
        "County & District Clerk" => {
            let name = if county_lower == "childress" {
                "county and district clerk"
            } else {
                "county clerk district clerk"
            };
            push_slug(&mut parts, county);
            push_slug(&mut parts, name);
            push_slug(&mut parts, candidate);
        }
        "Sheriff & County Tax Assessor-Collector" => {
            push_slug(&mut parts, county);
            push_slug(&mut parts, office_name);
            push_slug(&mut parts, candidate);
        }
        "Judge - County Civil Court at Law" | "Judge - County Court at Law" | "Judge - County Criminal Court of Appeals" => {
            push_slug(&mut parts, county);
            push_slug(&mut parts, office_name);
            if let Some(d) = district {
                push_slug(&mut parts, &format!("no {}", d));
            }
            push_slug(&mut parts, candidate);
        }
        "Judge - County Criminal Court at Law" => {
            let name = if county_lower == "tarrant" {
                "Judge - County Criminal Court"
            } else {
                office_name
            };
            push_slug(&mut parts, county);
            push_slug(&mut parts, name);
            if let Some(d) = district {
                push_slug(&mut parts, &format!("no {}", d));
            }
            push_slug(&mut parts, candidate);
        }
        "Judge - Probate Court" => {
            let name = if county_lower == "collin" || county_lower == "montgomery" {
                "Probate Court"
            } else if county_lower == "galveston" {
                "Judge - Probate Court at Law"
            } else if county_lower == "denton" {
                "Judge - County Probate Court at Law"
            } else {
                office_name
            };
            push_slug(&mut parts, county);
            push_slug(&mut parts, name);
            if let Some(d) = district {
                push_slug(&mut parts, &format!("no {}", d));
            }
            push_slug(&mut parts, candidate);
        }
        "County Commissioner" => {
            push_slug(&mut parts, county);
            push_slug(&mut parts, office_name);
            if let Some(d) = district {
                push_slug(&mut parts, &format!("precinct {}", d));
            }
            push_slug(&mut parts, candidate);
        }
        "Justice of the Peace" => {
            push_slug(&mut parts, county);
            push_slug(&mut parts, office_name);
            if let Some(d) = district {
                push_slug(&mut parts, &format!("precinct {}", d));
            }
            if let Some(s) = seat {
                push_slug(&mut parts, &format!("place {}", s));
            }
            push_slug(&mut parts, candidate);
        }
        "County Chair" => {
            push_slug(&mut parts, county);
            push_slug(&mut parts, office_name);
            push_slug(&mut parts, candidate);
        }
        "Precinct Chair" => {
            push_slug(&mut parts, county);
            push_slug(&mut parts, office_name);
            push_slug(&mut parts, "for pchr");
            if let Some(d) = district {
                let d_trimmed = d.trim_start_matches('0');
                push_slug(
                    &mut parts,
                    if d_trimmed.is_empty() { "0" } else { d_trimmed },
                );
            }
            if let Some(p) = party {
                push_slug(&mut parts, p);
            }
            push_slug(&mut parts, candidate);
        }
        "County School Trustee" => {
            push_slug(&mut parts, county);
            push_slug(&mut parts, "Harris County Department of Education");
            if let Some(d) = district {
                push_slug(&mut parts, &format!("place {}", d));
            }
            push_slug(&mut parts, candidate);
        }
        _ => {
            push_slug(&mut parts, county);
            push_slug(&mut parts, office_name);
            if let Some(d) = district {
                push_slug(&mut parts, &format!("precinct {}", d));
            }
            push_slug(&mut parts, candidate);
        }
    }
    parts.retain(|p| !p.is_empty());
    parts.join("-")
}

// ---------- Clarity ----------

#[derive(Debug, Clone)]
pub struct StgTxClarityResultRow {
    pub office_name: Option<String>,
    pub office_key: Option<String>,
    pub candidate_name: Option<String>,
    pub candidate_key: Option<String>,
    pub precincts_reporting: Option<i64>,
    pub precincts_total: Option<i64>,
    pub votes_for_candidate: Option<i64>,
    pub total_votes: Option<i64>,
    pub total_voters: Option<i64>,
    pub party: Option<String>,
    pub race_type: Option<String>,
    pub election_year: Option<i32>,
    pub ref_key: String,
    pub source_file: Option<String>,
    pub county: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ClarityCsvRow {
    #[serde(rename = "contest name")]
    contest_name: Option<String>,
    #[serde(rename = "choice name")]
    choice_name: Option<String>,
    #[serde(rename = "party name")]
    party_name: Option<String>,
    #[serde(rename = "total votes")]
    total_votes: Option<i64>,
    #[serde(rename = "ballots cast")]
    ballots_cast: Option<i64>,
    #[serde(rename = "percent of votes")]
    percent_of_votes: Option<String>,
    #[serde(rename = "num Precinct total")]
    num_precinct_total: Option<i64>,
    #[serde(rename = "num Precinct rptg")]
    num_precinct_rptg: Option<i64>,
    county: Option<String>,
}

fn total_votes_from_percent(candidate_votes: Option<i64>, percent_of_votes: Option<&str>) -> Option<i64> {
    let votes = candidate_votes?;
    let pct_str = percent_of_votes?.trim().trim_end_matches('%').trim();
    if pct_str.is_empty() {
        return None;
    }
    let pct: f64 = pct_str.parse().ok()?;
    if !pct.is_finite() || pct <= 0.0 {
        return None;
    }
    let total = (votes as f64 * 100.0 / pct).round();
    if total.is_finite() && total >= 0.0 && total <= i64::MAX as f64 {
        Some(total as i64)
    } else {
        None
    }
}

pub fn parse_clarity_csv(
    csv_path: &Path,
    source_file: &str,
    election_year: Option<i32>,
) -> Result<Vec<StgTxClarityResultRow>, Box<dyn std::error::Error + Send + Sync>> {
    let year = election_year.unwrap_or(DEFAULT_ELECTION_YEAR);
    let bytes = fs::read(csv_path)?;
    let content = decode_csv_bytes_to_utf8(&bytes);
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(Cursor::new(content.as_bytes()));
    let mut rows = Vec::new();
    for result in rdr.deserialize() {
        let raw: ClarityCsvRow = result?;
        let contest_name_raw = raw.contest_name.as_deref().unwrap_or("").trim();
        if contest_name_raw.is_empty() {
            continue;
        }
        let normalized_contest = normalize_contest_name(contest_name_raw);
        let rule = match find_matching_county_office(&normalized_contest) {
            Some(r) => r,
            None => continue,
        };
        let parsed = parse_contest_name_for_office(rule, &normalized_contest);
        let county_name = raw.county.as_deref().filter(|s| !s.trim().is_empty());
        let candidate_name = raw
            .choice_name
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let ref_key = build_ref_key_for_county_race(
            rule.populist_office_name,
            &parsed,
            county_name,
            candidate_name.as_deref(),
            raw.party_name.as_deref(),
            year,
        );
        let total_votes = total_votes_from_percent(raw.total_votes, raw.percent_of_votes.as_deref());
        rows.push(StgTxClarityResultRow {
            office_name: Some(rule.populist_office_name.to_string()),
            office_key: None,
            candidate_name,
            candidate_key: None,
            precincts_reporting: raw.num_precinct_rptg,
            precincts_total: raw.num_precinct_total,
            votes_for_candidate: raw.total_votes,
            total_votes,
            total_voters: None,
            party: raw.party_name.filter(|s| !s.trim().is_empty()),
            race_type: None,
            election_year: Some(year),
            ref_key,
            source_file: Some(source_file.to_string()),
            county: raw.county.filter(|s| !s.trim().is_empty()),
        });
    }
    Ok(rows)
}

async fn insert_clarity_staging_rows(
    pool: &PgPool,
    rows: &[StgTxClarityResultRow],
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    if rows.is_empty() {
        return Ok(0);
    }
    let mut count = 0u64;
    for row in rows {
        sqlx::query(
            r#"
            INSERT INTO ingest_staging.stg_tx_results_clarity (
                office_name, office_key, candidate_name, candidate_key,
                precincts_reporting, precincts_total, votes_for_candidate, total_votes, total_voters,
                party, race_type, election_year, ref_key, source_file, county
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(&row.office_name)
        .bind(&row.office_key)
        .bind(&row.candidate_name)
        .bind(&row.candidate_key)
        .bind(row.precincts_reporting)
        .bind(row.precincts_total)
        .bind(row.votes_for_candidate)
        .bind(row.total_votes)
        .bind(row.total_voters)
        .bind(&row.party)
        .bind(&row.race_type)
        .bind(row.election_year)
        .bind(&row.ref_key)
        .bind(&row.source_file)
        .bind(&row.county)
        .execute(pool)
        .await?;
        count += 1;
    }
    Ok(count)
}

pub async fn process_clarity_csv(
    pool: &PgPool,
    csv_path: &Path,
    source_file: &str,
    election_year: Option<i32>,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let rows = parse_clarity_csv(csv_path, source_file, election_year)?;
    insert_clarity_staging_rows(pool, &rows).await
}

// ---------- Hart ----------

const SUMMARY_CHOICES_SKIP: &[&str] = &["Cast Votes", "Undervotes", "Overvotes"];

#[derive(Debug, Clone)]
pub struct StgTxHartResultRow {
    pub office_name: Option<String>,
    pub office_key: Option<String>,
    pub candidate_name: Option<String>,
    pub candidate_key: Option<String>,
    pub precincts_reporting: Option<i64>,
    pub precincts_total: Option<i64>,
    pub votes_for_candidate: Option<i64>,
    pub total_votes: Option<i64>,
    pub total_voters: Option<i64>,
    pub party: Option<String>,
    pub race_type: Option<String>,
    pub election_year: Option<i32>,
    pub ref_key: String,
    pub source_file: Option<String>,
    pub county: Option<String>,
}

pub fn parse_hart_csv(
    csv_path: &Path,
    source_file: &str,
    election_year: Option<i32>,
) -> Result<Vec<StgTxHartResultRow>, Box<dyn std::error::Error + Send + Sync>> {
    let year = election_year.unwrap_or(DEFAULT_ELECTION_YEAR);
    let bytes = fs::read(csv_path)?;
    let content = decode_csv_bytes_to_utf8(&bytes);
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(Cursor::new(content.as_bytes()));
    let headers = rdr.headers()?.clone();
    let header_map: HashMap<String, usize> = headers
        .iter()
        .enumerate()
        .map(|(i, h)| (h.trim().to_string(), i))
        .collect();
    let race_idx = header_map
        .get("Race")
        .copied()
        .ok_or_else(|| "Hart CSV missing 'Race' column".to_string())?;
    let choice_idx = header_map
        .get("Choice")
        .copied()
        .ok_or_else(|| "Hart CSV missing 'Choice' column".to_string())?;
    let total_idx = header_map.get("Total").copied();
    let total_cast_votes_idx = header_map.get("Total Cast Votes").copied();
    let precincts_counted_idx = header_map.get("Precincts Counted").copied();
    let precincts_total_idx = header_map.get("Precincts Total").copied();
    let county_idx = header_map.get("County").copied();
    let party_idx = header_map.get("Party").copied();
    let mut rows = Vec::new();
    let mut record = StringRecord::new();
    while rdr.read_record(&mut record)? {
        let race_raw = record.get(race_idx).map(|s| s.trim()).unwrap_or("");
        if race_raw.is_empty() {
            continue;
        }
        let choice_raw = record.get(choice_idx).map(|s| s.trim()).unwrap_or("");
        if choice_raw.is_empty() || SUMMARY_CHOICES_SKIP.iter().any(|c| choice_raw.eq(*c)) {
            continue;
        }
        let normalized_contest = normalize_contest_name(race_raw);
        let rule = match find_matching_county_office(&normalized_contest) {
            Some(r) => r,
            None => continue,
        };
        let parsed = parse_contest_name_for_office(rule, &normalized_contest);
        let county_name = county_idx
            .and_then(|i| record.get(i))
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let county_ref = county_name.as_deref();
        let candidate_name = Some(choice_raw.to_string());
        let party = party_idx
            .and_then(|i| record.get(i))
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let votes_for_candidate = total_idx.and_then(|i| record.get(i)).and_then(|s| {
            s.trim().replace(',', "").parse::<i64>().ok()
        });
        let total_votes = total_cast_votes_idx.and_then(|i| record.get(i)).and_then(|s| {
            s.trim().replace(',', "").parse::<i64>().ok()
        });
        let precincts_reporting = precincts_counted_idx
            .and_then(|i| record.get(i))
            .and_then(|s| s.trim().replace(',', "").parse::<i64>().ok());
        let precincts_total = precincts_total_idx
            .and_then(|i| record.get(i))
            .and_then(|s| s.trim().replace(',', "").parse::<i64>().ok());
        let ref_key = build_ref_key_for_county_race(
            rule.populist_office_name,
            &parsed,
            county_ref,
            candidate_name.as_deref(),
            party.as_deref(),
            year,
        );
        rows.push(StgTxHartResultRow {
            office_name: Some(rule.populist_office_name.to_string()),
            office_key: None,
            candidate_name,
            candidate_key: None,
            precincts_reporting,
            precincts_total,
            votes_for_candidate,
            total_votes,
            total_voters: None,
            party,
            race_type: None,
            election_year: Some(year),
            ref_key,
            source_file: Some(source_file.to_string()),
            county: county_name,
        });
    }
    Ok(rows)
}

async fn insert_hart_staging_rows(
    pool: &PgPool,
    rows: &[StgTxHartResultRow],
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    if rows.is_empty() {
        return Ok(0);
    }
    let mut count = 0u64;
    for row in rows {
        sqlx::query(
            r#"
            INSERT INTO ingest_staging.stg_tx_results_hart (
                office_name, office_key, candidate_name, candidate_key,
                precincts_reporting, precincts_total, votes_for_candidate, total_votes, total_voters,
                party, race_type, election_year, ref_key, source_file, county
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(&row.office_name)
        .bind(&row.office_key)
        .bind(&row.candidate_name)
        .bind(&row.candidate_key)
        .bind(row.precincts_reporting)
        .bind(row.precincts_total)
        .bind(row.votes_for_candidate)
        .bind(row.total_votes)
        .bind(row.total_voters)
        .bind(&row.party)
        .bind(&row.race_type)
        .bind(row.election_year)
        .bind(&row.ref_key)
        .bind(&row.source_file)
        .bind(&row.county)
        .execute(pool)
        .await?;
        count += 1;
    }
    Ok(count)
}

pub async fn process_hart_csv(
    pool: &PgPool,
    csv_path: &Path,
    source_file: &str,
    election_year: Option<i32>,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let rows = parse_hart_csv(csv_path, source_file, election_year)?;
    insert_hart_staging_rows(pool, &rows).await
}

// ---------- Other ----------

pub const TX_OTHER_DATA_DIR: &str = "data/tx/counties/other";

pub fn other_data_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    Path::new(&manifest_dir).join(TX_OTHER_DATA_DIR)
}

#[derive(Debug, Clone)]
pub struct StgTxOtherResultRow {
    pub office_name: Option<String>,
    pub office_key: Option<String>,
    pub candidate_name: Option<String>,
    pub candidate_key: Option<String>,
    pub precincts_reporting: Option<i64>,
    pub precincts_total: Option<i64>,
    pub votes_for_candidate: Option<i64>,
    pub total_votes: Option<i64>,
    pub total_voters: Option<i64>,
    pub party: Option<String>,
    pub race_type: Option<String>,
    pub election_year: Option<i32>,
    pub ref_key: String,
    pub source_file: Option<String>,
    pub county: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct OtherCsvRow {
    #[serde(rename = "contest name")]
    contest_name: Option<String>,
    #[serde(rename = "choice name")]
    choice_name: Option<String>,
    #[serde(rename = "party name")]
    party_name: Option<String>,
    #[serde(rename = "votes for candidate")]
    votes_for_candidate: Option<i64>,
    #[serde(rename = "total votes")]
    total_votes: Option<i64>,
    #[serde(rename = "ballots cast")]
    ballots_cast: Option<i64>,
    #[serde(rename = "percent of votes")]
    percent_of_votes: Option<String>,
    #[serde(rename = "num Precinct total")]
    num_precinct_total: Option<i64>,
    #[serde(rename = "num Precinct rptg")]
    num_precinct_rptg: Option<i64>,
    county: Option<String>,
}

pub fn parse_other_csv(
    csv_path: &Path,
    source_file: &str,
    election_year: Option<i32>,
) -> Result<Vec<StgTxOtherResultRow>, Box<dyn std::error::Error + Send + Sync>> {
    let year = election_year.unwrap_or(DEFAULT_ELECTION_YEAR);
    let bytes = fs::read(csv_path)?;
    let content = decode_csv_bytes_to_utf8(&bytes);
    let mut rdr = ReaderBuilder::new()
        .has_headers(true)
        .trim(csv::Trim::All)
        .from_reader(Cursor::new(content.as_bytes()));
    let mut rows = Vec::new();
    for result in rdr.deserialize() {
        let raw: OtherCsvRow = result?;
        let contest_name_raw = raw.contest_name.as_deref().unwrap_or("").trim();
        if contest_name_raw.is_empty() {
            continue;
        }
        let normalized_contest = normalize_contest_name(contest_name_raw);
        let rule = match find_matching_county_office(&normalized_contest) {
            Some(r) => r,
            None => continue,
        };
        let parsed = parse_contest_name_for_office(rule, &normalized_contest);
        let county_name = raw.county.as_deref().filter(|s| !s.trim().is_empty());
        let candidate_name = raw
            .choice_name
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        let ref_key = build_ref_key_for_county_race(
            rule.populist_office_name,
            &parsed,
            county_name,
            candidate_name.as_deref(),
            raw.party_name.as_deref(),
            year,
        );
        let total_votes = raw.total_votes
            .or_else(|| total_votes_from_percent(raw.votes_for_candidate, raw.percent_of_votes.as_deref()));
        rows.push(StgTxOtherResultRow {
            office_name: Some(rule.populist_office_name.to_string()),
            office_key: None,
            candidate_name,
            candidate_key: None,
            precincts_reporting: raw.num_precinct_rptg,
            precincts_total: raw.num_precinct_total,
            votes_for_candidate: raw.votes_for_candidate,
            total_votes,
            total_voters: None,
            party: raw.party_name.filter(|s| !s.trim().is_empty()),
            race_type: None,
            election_year: Some(year),
            ref_key,
            source_file: Some(source_file.to_string()),
            county: raw.county.filter(|s| !s.trim().is_empty()),
        });
    }
    Ok(rows)
}

async fn insert_other_staging_rows(
    pool: &PgPool,
    rows: &[StgTxOtherResultRow],
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    if rows.is_empty() {
        return Ok(0);
    }
    let mut count = 0u64;
    for row in rows {
        sqlx::query(
            r#"
            INSERT INTO ingest_staging.stg_tx_results_other (
                office_name, office_key, candidate_name, candidate_key,
                precincts_reporting, precincts_total, votes_for_candidate, total_votes, total_voters,
                party, race_type, election_year, ref_key, source_file, county
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(&row.office_name)
        .bind(&row.office_key)
        .bind(&row.candidate_name)
        .bind(&row.candidate_key)
        .bind(row.precincts_reporting)
        .bind(row.precincts_total)
        .bind(row.votes_for_candidate)
        .bind(row.total_votes)
        .bind(row.total_voters)
        .bind(&row.party)
        .bind(&row.race_type)
        .bind(row.election_year)
        .bind(&row.ref_key)
        .bind(&row.source_file)
        .bind(&row.county)
        .execute(pool)
        .await?;
        count += 1;
    }
    Ok(count)
}

pub async fn process_other_csv(
    pool: &PgPool,
    csv_path: &Path,
    source_file: &str,
    election_year: Option<i32>,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let rows = parse_other_csv(csv_path, source_file, election_year)?;
    insert_other_staging_rows(pool, &rows).await
}
