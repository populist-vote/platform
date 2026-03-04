//! Processor for TX county Hart election results CSV files.
//!
//! Reads PDF-style CSVs produced by tx_hart_results_pdf_processor (Race, Choice, Party,
//! per-method vote columns, Precincts Counted/Total, County), maps rows to staging shape
//! and inserts into ingest_staging.stg_tx_results_hart.
//! Contest name parsing starts from the same rules as the Clarity processor; customize
//! for Hart contest name format as needed.

use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::Path;

use csv::{ReaderBuilder, StringRecord};
use sqlx::PgPool;
use slugify::slugify;

use crate::extractors::tx::office::{extract_office_district, extract_office_seat};
use crate::util::decode_csv_bytes_to_utf8;

/// Default election year when not derivable from CSV (for ref_key generation).
const DEFAULT_ELECTION_YEAR: i32 = 2026;

/// Summary choice names that are skipped (not candidate rows).
const SUMMARY_CHOICES_SKIP: &[&str] = &["Cast Votes", "Undervotes", "Overvotes"];

/// County office match rule: populist name, strings to match, and strings that disqualify the match.
/// If contest_name contains any match_substring (case-insensitive), the rule is a candidate.
/// If contest_name also contains any exclude_substrings (case-insensitive), the rule is skipped.
struct CountyOfficeRule {
    populist_office_name: &'static str,
    /// Match if normalized contest_name contains any of these (case-insensitive).
    match_substrings: &'static [&'static str],
    /// Do not match if normalized contest_name contains any of these (case-insensitive).
    exclude_substrings: &'static [&'static str],
}

/// County-level office matching rules. Order matters: first match wins.
/// Initially copied from Clarity; edit for Hart contest name formatting.
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

/// Normalize contest name: strip party/instruction suffixes and prefixes in a fixed order.
/// Order: (1) " - Vote for none or one", (2) " - Republican Party" / " - Democratic Party",
/// (3) " (D)" / " (R)", then "REP - " / "DEM - " from the front (case-insensitive), then "(Vote for 1)" from end; also remove "unexpired term".
fn normalize_contest_name(s: &str) -> String {
    let mut s: String = remove_substring_ignore_ascii_case(s.trim(), "unexpired term")
        .trim()
        .trim_end_matches(|c| c == ' ' || c == '-')
        .into();

    // 1. Remove " - Vote for none or one" from end (case-insensitive)
    let suffix1 = " - Vote for none or one";
    if s.len() >= suffix1.len() && s[s.len() - suffix1.len()..].eq_ignore_ascii_case(suffix1) {
        s = s[..s.len() - suffix1.len()].trim_end().to_string();
    }

    // 2. Remove " - Republican Party" or " - Democratic Party" from end (case-insensitive)
    let suffix_rep = " - Republican Party";
    let suffix_dem = " - Democratic Party";
    if s.len() >= suffix_rep.len() && s[s.len() - suffix_rep.len()..].eq_ignore_ascii_case(suffix_rep) {
        s = s[..s.len() - suffix_rep.len()].trim_end().to_string();
    } else if s.len() >= suffix_dem.len() && s[s.len() - suffix_dem.len()..].eq_ignore_ascii_case(suffix_dem) {
        s = s[..s.len() - suffix_dem.len()].trim_end().to_string();
    }

    // 3. Remove " (D)" or " (R)" from end (case-insensitive)
    let suffix_d = " (D)";
    let suffix_r = " (R)";
    if s.len() >= suffix_d.len() && s[s.len() - suffix_d.len()..].eq_ignore_ascii_case(suffix_d) {
        s = s[..s.len() - suffix_d.len()].trim_end().to_string();
    } else if s.len() >= suffix_r.len() && s[s.len() - suffix_r.len()..].eq_ignore_ascii_case(suffix_r) {
        s = s[..s.len() - suffix_r.len()].trim_end().to_string();
    }

    // 4. Remove "REP - " or "DEM - " from front (case-insensitive)
    let prefix_rep = "REP - ";
    let prefix_dem = "DEM - ";
    if s.len() >= prefix_rep.len() && s[..prefix_rep.len()].eq_ignore_ascii_case(prefix_rep) {
        s = s[prefix_rep.len()..].trim_start().to_string();
    } else if s.len() >= prefix_dem.len() && s[..prefix_dem.len()].eq_ignore_ascii_case(prefix_dem) {
        s = s[prefix_dem.len()..].trim_start().to_string();
    }

    // 5. Remove "(Vote for 1)" from end (case-insensitive), for Clarity-style names
    let suffix_vote1 = "(Vote for 1)";
    if s.len() >= suffix_vote1.len() && s[s.len() - suffix_vote1.len()..].eq_ignore_ascii_case(suffix_vote1) {
        s = s[..s.len() - suffix_vote1.len()].trim_end().to_string();
    }

    s
}

/// Parsed bits from contest_name for ref_key building.
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

/// One row written to ingest_staging.stg_tx_results_hart.
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

/// Parse a single Hart PDF-style CSV into staging rows for ingest_staging.stg_tx_results_hart.
/// Skips summary rows (Cast Votes, Undervotes, Overvotes). Only rows whose Race matches
/// a county-level office in COUNTY_OFFICE_MATCH_RULES are included.
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

    let race_idx = header_map.get("Race").copied();
    let choice_idx = header_map.get("Choice").copied();
    let party_idx = header_map.get("Party").copied();
    // "Total" = candidate's total votes (votes_for_candidate)
    let total_idx = header_map.get("Total").copied();
    // "Total Cast Votes" = race ballots cast (total_votes)
    let total_cast_votes_idx = header_map.get("Total Cast Votes").copied();
    let precincts_counted_idx = header_map.get("Precincts Counted").copied();
    let precincts_total_idx = header_map.get("Precincts Total").copied();
    let county_idx = header_map.get("County").copied();

    let race_idx = match race_idx {
        Some(i) => i,
        None => return Err("Hart CSV missing 'Race' column".into()),
    };
    let choice_idx = match choice_idx {
        Some(i) => i,
        None => return Err("Hart CSV missing 'Choice' column".into()),
    };

    let mut rows = Vec::new();
    let mut record = StringRecord::new();
    while rdr.read_record(&mut record)? {
        let race_raw = record
            .get(race_idx)
            .map(|s| s.trim())
            .unwrap_or("");
        if race_raw.is_empty() {
            continue;
        }

        let choice_raw = record.get(choice_idx).map(|s| s.trim()).unwrap_or("");
        if choice_raw.is_empty() {
            continue;
        }
        if SUMMARY_CHOICES_SKIP.iter().any(|c| choice_raw.eq(*c)) {
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
        let candidate_name = if choice_raw.is_empty() {
            None
        } else {
            Some(choice_raw.to_string())
        };
        let party = party_idx
            .and_then(|i| record.get(i))
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let votes_for_candidate = total_idx.and_then(|i| record.get(i)).and_then(|s| {
            let s = s.trim().replace(',', "");
            s.parse::<i64>().ok()
        });
        let total_votes = total_cast_votes_idx.and_then(|i| record.get(i)).and_then(|s| {
            let s = s.trim().replace(',', "");
            s.parse::<i64>().ok()
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

/// Insert a batch of rows into ingest_staging.stg_tx_results_hart.
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

/// Parse the Hart CSV at `csv_path` and insert all rows into ingest_staging.stg_tx_results_hart.
/// The staging table must already exist (call ensure_hart_staging_table in the scraper once per run).
pub async fn process_hart_csv(
    pool: &PgPool,
    csv_path: &Path,
    source_file: &str,
    election_year: Option<i32>,
) -> Result<u64, Box<dyn std::error::Error + Send + Sync>> {
    let rows = parse_hart_csv(csv_path, source_file, election_year)?;
    insert_hart_staging_rows(pool, &rows).await
}
