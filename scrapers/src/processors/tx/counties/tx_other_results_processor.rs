//! Processor for TX county "other" election results CSV files.
//!
//! Reads CSV files from data/tx/counties/other (contest name, choice name, party, county, etc.),
//! maps rows to staging shape and inserts into ingest_staging.stg_tx_results_other.
//! Same CSV format and county-level matching rules as Hart/Clarity; contest normalization and
//! ref_key logic aligned with tx_hart_results_processor.

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use csv::ReaderBuilder;
use sqlx::PgPool;
use slugify::slugify;

use crate::extractors::tx::office::{extract_office_district, extract_office_seat};
use crate::util::decode_csv_bytes_to_utf8;

/// Default election year when not derivable from CSV (for ref_key generation).
const DEFAULT_ELECTION_YEAR: i32 = 2026;

/// Directory for TX "other" county results CSVs (relative to scrapers crate root).
pub const TX_OTHER_DATA_DIR: &str = "data/tx/counties/other";

/// Return the path to the TX other data directory.
pub fn other_data_path() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    Path::new(&manifest_dir).join(TX_OTHER_DATA_DIR)
}

/// County office match rule: populist name, strings to match, and strings that disqualify the match.
struct CountyOfficeRule {
    populist_office_name: &'static str,
    match_substrings: &'static [&'static str],
    exclude_substrings: &'static [&'static str],
}

const COUNTY_OFFICE_MATCH_RULES: &[CountyOfficeRule] = &[
    CountyOfficeRule {
        populist_office_name: "Judge - County Criminal Court of Appeals",
        match_substrings: &["Judge, County Criminal Court of Appeals"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "Judge - County Criminal Court at Law",
        match_substrings: &["Judge, County Criminal Court"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "Judge - County Court at Law",
        match_substrings: &["Judge, County Court at Law", "Judge, County Court-at-Law"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "Judge - Probate Court",
        match_substrings: &["Probate Court"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "Criminal District Judge",
        match_substrings: &["Criminal District Judge"],
        exclude_substrings: &["Judicial District"],
    },
    CountyOfficeRule {
        populist_office_name: "Criminal District Attorney",
        match_substrings: &["Criminal District Attorney"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "County Judge",
        match_substrings: &["County Judge"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "Judge - County Civil Court at Law",
        match_substrings: &["Judge, County Civil Court at Law"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "County Attorney",
        match_substrings: &["County Attorney"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "County & District Clerk",
        match_substrings: &["County and District Clerk", "District and County Clerk"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "District Clerk",
        match_substrings: &["District Clerk"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "County Clerk",
        match_substrings: &["County Clerk"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "Sheriff & County Tax Assessor-Collector",
        match_substrings: &["Sheriff and Tax Assessor-Collector"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "Sheriff",
        match_substrings: &["Sheriff"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "County Tax Assessor-Collector",
        match_substrings: &["Tax Assessor-Collector"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "County Treasurer",
        match_substrings: &["County Treasurer"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "County Surveyor",
        match_substrings: &["County Surveyor"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "County School Trustee",
        match_substrings: &["County School Trustee"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "County Commissioner",
        match_substrings: &["County Commissioner", "Commissioner"],
        exclude_substrings: &["Railroad", "Agriculture", "General Land Office"],
    },
    CountyOfficeRule {
        populist_office_name: "Justice of the Peace",
        match_substrings: &["Justice of the Peace", "JOP"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "County Constable",
        match_substrings: &["Constable"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "Precinct Chair",
        match_substrings: &["Precinct Chair"],
        exclude_substrings: &[],
    },
    CountyOfficeRule {
        populist_office_name: "County Chair",
        match_substrings: &["County Chair", "Party Chairman", "Party Chair"],
        exclude_substrings: &[],
    },
];

fn find_matching_county_office(normalized_contest_name: &str) -> Option<&'static CountyOfficeRule> {
    let contest_lower = normalized_contest_name.to_lowercase();
    for rule in COUNTY_OFFICE_MATCH_RULES {
        if rule.match_substrings.is_empty() {
            continue;
        }
        let matched = rule.match_substrings.iter().any(|s| {
            contest_lower.contains(&s.to_lowercase())
        });
        if !matched {
            continue;
        }
        let excluded = rule.exclude_substrings.iter().any(|s| {
            contest_lower.contains(&s.to_lowercase())
        });
        if excluded {
            continue;
        }
        return Some(rule);
    }
    None
}

/// Remove all (case-insensitive) occurrences of `phrase` from `s`.
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

/// Normalize contest name: strip party/instruction suffixes and prefixes (same order as Hart); also remove "unexpired term".
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

/// One row written to ingest_staging.stg_tx_results_other.
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

#[derive(Debug, Clone)]
struct ParsedContestName {
    office_name: String,
    district: Option<String>,
    seat: Option<String>,
}

fn parse_contest_name_for_office(rule: &CountyOfficeRule, contest_name: &str) -> ParsedContestName {
    let (seat, _stripped) = extract_office_seat(contest_name);
    let district = extract_office_district(&_stripped, None);
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
    let district = parsed.district.as_deref().map(|s| s.trim()).filter(|s| !s.is_empty());
    let seat = parsed.seat.as_deref().map(|s| s.trim()).filter(|s| !s.is_empty());
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
        "Judge - County Civil Court at Law" => {
            push_slug(&mut parts, county);
            push_slug(&mut parts, office_name);
            if let Some(d) = district {
                push_slug(&mut parts, &format!("no {}", d));
            }
            push_slug(&mut parts, candidate);
        }
        "Judge - County Court at Law" => {
            push_slug(&mut parts, county);
            push_slug(&mut parts, office_name);
            if let Some(d) = district {
                push_slug(&mut parts, &format!("no {}", d));
            }
            push_slug(&mut parts, candidate);
        }
        "Judge - County Criminal Court of Appeals" => {
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
            let name = if county_lower == "collin" {
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
                push_slug(&mut parts, if d_trimmed.is_empty() { "0" } else { d_trimmed });
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
                push_slug(&mut parts, &format!("Position {}", d));
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

/// One row from an "other" county results CSV (same column names as Clarity-style).
#[derive(Debug, serde::Deserialize)]
struct OtherCsvRow {
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

/// Compute race total from candidate votes and percent: total_votes = candidate_votes / (percent / 100).
/// Same logic as tx_clarity_results_processor. Returns None if percent is missing, zero, or unparseable.
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

/// Parse a single "other" CSV file into staging rows for ingest_staging.stg_tx_results_other.
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

        let total_votes = total_votes_from_percent(raw.total_votes, raw.percent_of_votes.as_deref());

        rows.push(StgTxOtherResultRow {
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

/// Parse the "other" CSV at `csv_path` and insert all rows into ingest_staging.stg_tx_results_other.
pub async fn process_other_csv(
    pool: &PgPool,
    csv_path: &Path,
    source_file: &str,
    election_year: Option<i32>,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let rows = parse_other_csv(csv_path, source_file, election_year)?;
    insert_other_staging_rows(pool, &rows).await
}
