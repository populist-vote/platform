//! Texas office extractors: parse raw office title strings (e.g. from candidate filings)
//! into office_title, political_scope, election_scope, district_type, district, seat, county.
//!
//! Patterns aligned with populist-office-titles-map.csv and MN office extractors.

use std::sync::OnceLock;

use regex::Regex;

use crate::extractors::{default_capture, owned_capture};

/// Strip "COUNTY_NAME - " prefix from raw title; returns (county_name_if_any, rest).
/// Only accepts the prefix when it matches a valid Texas county (TEXAS_COUNTIES), same as strip_county_suffix.
/// Example: "KLEBERG - COUNTY COMMISSIONER PRECINCT 4" -> (Some("Kleberg"), "COUNTY COMMISSIONER PRECINCT 4")
fn strip_county_prefix(input: &str) -> (Option<String>, &str) {
    let prefix_re = TX_COUNTY_PREFIX_REGEX.get_or_init(|| {
        Regex::new(r"^\s*([A-Z][A-Za-z\s]+?)\s*-\s*(.*)$").unwrap()
    });
    if let Some(caps) = prefix_re.captures(input) {
        if let (Some(county), Some(rest)) = (caps.get(1), caps.get(2)) {
            let county_str = county.as_str().trim();
            let rest_str = rest.as_str().trim();
            let county_norm = normalize_county_name(county_str);
            if !rest_str.is_empty() && county_str.len() >= 2 && is_valid_texas_county(&county_norm) {
                let county_title_case = title_case(&county_norm);
                return (Some(county_title_case), rest_str);
            }
        }
    }
    (None, input.trim())
}

/// Collapse internal whitespace to single spaces and trim.
fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Strip " COUNTY_NAME COUNTY" (and optional trailing text like "NUMBER 1" or "#1") from raw title.
/// Returns (county_name_if_any, rest_with_normalized_whitespace).
/// Iterates over TEXAS_COUNTIES (longest names first) and uses a string match so multi-word
/// counties (e.g. "Jim Hogg", "Deaf Smith") are matched correctly without regex.
/// Examples:
///   "CRIMINAL DISTRICT ATTORNEY JIM HOGG COUNTY" -> (Some("Jim Hogg"), "CRIMINAL DISTRICT ATTORNEY")
///   "CRIMINAL DISTRICT JUDGE #1 TARRANT COUNTY" -> (Some("Tarrant"), "CRIMINAL DISTRICT JUDGE #1")
///   "CRIMINAL DISTRICT JUDGE DALLAS COUNTY NUMBER 1" -> (Some("Dallas"), "CRIMINAL DISTRICT JUDGE NUMBER 1")
fn strip_county_suffix(input: &str) -> (Option<String>, String) {
    let trimmed = input.trim();
    let input_upper = trimmed.to_uppercase();
    let mut counties: Vec<&str> = TEXAS_COUNTIES.iter().copied().collect();
    counties.sort_by_key(|c| std::cmp::Reverse(c.len()));

    for county in counties {
        let needle = format!(" {} COUNTY", county);
        let needle_upper = needle.to_uppercase();
        if let Some(pos) = input_upper.find(&needle_upper) {
            let prefix = trimmed[..pos].trim();
            let after = trimmed[pos + needle.len()..].trim();
            let rest = if after.is_empty() {
                prefix.to_string()
            } else {
                format!("{} {}", prefix, after)
            };
            return (Some(county.to_string()), normalize_whitespace(&rest));
        }
    }
    (None, normalize_whitespace(trimmed))
}

fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().chain(c.flat_map(|c| c.to_lowercase())).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

static TX_COUNTY_PREFIX_REGEX: OnceLock<Regex> = OnceLock::new();
static TX_DISTRICT_EXTRACTORS: OnceLock<Vec<Regex>> = OnceLock::new();
static TX_SEAT_EXTRACTORS: OnceLock<Regex> = OnceLock::new();

/// All 254 Texas county names (title case), for validating extracted county strings.
const TEXAS_COUNTIES: [&str; 254] = [
    "Anderson", "Andrews", "Angelina", "Aransas", "Archer", "Armstrong", "Atascosa", "Austin",
    "Bailey", "Bandera", "Bastrop", "Baylor", "Bee", "Bell", "Bexar", "Blanco", "Borden", "Bosque",
    "Bowie", "Brazoria", "Brazos", "Brewster", "Briscoe", "Brooks", "Brown", "Burleson", "Burnet",
    "Caldwell", "Calhoun", "Callahan", "Cameron", "Camp", "Carson", "Cass", "Castro", "Chambers",
    "Cherokee", "Childress", "Clay", "Cochran", "Coke", "Coleman", "Collin", "Collingsworth",
    "Colorado", "Comal", "Comanche", "Concho", "Cooke", "Coryell", "Cottle", "Crane", "Crockett",
    "Crosby", "Culberson", "Dallam", "Dallas", "Dawson", "Deaf Smith", "Delta", "Denton", "DeWitt",
    "Dickens", "Dimmit", "Donley", "Duval", "Eastland", "Ector", "Edwards", "Ellis", "El Paso",
    "Erath", "Falls", "Fannin", "Fayette", "Fisher", "Floyd", "Foard", "Fort Bend", "Franklin",
    "Freestone", "Frio", "Gaines", "Galveston", "Garza", "Gillespie", "Glasscock", "Goliad",
    "Gonzales", "Gray", "Grayson", "Gregg", "Grimes", "Guadalupe", "Hale", "Hall", "Hamilton",
    "Hansford", "Hardeman", "Hardin", "Harris", "Harrison", "Hartley", "Haskell", "Hays",
    "Hemphill", "Henderson", "Hidalgo", "Hill", "Hockley", "Hood", "Hopkins", "Houston", "Howard",
    "Hudspeth", "Hunt", "Hutchinson", "Irion", "Jack", "Jackson", "Jasper", "Jeff Davis",
    "Jefferson", "Jim Hogg", "Jim Wells", "Johnson", "Jones", "Karnes", "Kaufman", "Kendall",
    "Kenedy", "Kent", "Kerr", "Kimble", "King", "Kinney", "Kleberg", "Knox", "La Salle", "Lamar",
    "Lamb", "Lampasas", "Lavaca", "Lee", "Leon", "Liberty", "Limestone", "Lipscomb", "Live Oak",
    "Llano", "Loving", "Lubbock", "Lynn", "Madison", "Marion", "Martin", "Mason", "Matagorda",
    "Maverick", "McCulloch", "McLennan", "McMullen", "Medina", "Menard", "Midland", "Milam", "Mills",
    "Mitchell", "Montague", "Montgomery", "Moore", "Morris", "Motley", "Nacogdoches", "Navarro",
    "Newton", "Nolan", "Nueces", "Ochiltree", "Oldham", "Orange", "Palo Pinto", "Panola", "Parker",
    "Parmer", "Pecos", "Polk", "Potter", "Presidio", "Rains", "Randall", "Reagan", "Real", "Red River",
    "Reeves", "Refugio", "Roberts", "Robertson", "Rockwall", "Runnels", "Rusk", "Sabine",
    "San Augustine", "San Jacinto", "San Patricio", "San Saba", "Schleicher", "Scurry", "Shackelford",
    "Shelby", "Sherman", "Smith", "Somervell", "Starr", "Stephens", "Sterling", "Stonewall",
    "Sutton", "Swisher", "Tarrant", "Taylor", "Terrell", "Terry", "Throckmorton", "Titus", "Tom Green",
    "Travis", "Trinity", "Tyler", "Upshur", "Upton", "Uvalde", "Val Verde", "Van Zandt", "Victoria",
    "Walker", "Waller", "Ward", "Washington", "Webb", "Wharton", "Wheeler", "Wichita", "Wilbarger",
    "Willacy", "Williamson", "Wilson", "Winkler", "Wise", "Wood", "Yoakum", "Young", "Zapata", "Zavala",
];

fn is_valid_texas_county(county: &str) -> bool {
    let county_trimmed = county.trim();
    TEXAS_COUNTIES
        .iter()
        .any(|c| c.eq_ignore_ascii_case(county_trimmed))
}

/// Normalize extracted county string to match canonical formatting (e.g. "La Salle" not "Lasalle").
fn normalize_county_name(county: &str) -> String {
    if county.eq_ignore_ascii_case("lasalle") {
        "La Salle".to_string()
    } else {
        county.to_string()
    }
}

/// Office names for which county may appear as a suffix (" X COUNTY") in the raw title.
const OFFICE_NAMES_ALLOWING_COUNTY_SUFFIX: &[&str] = &[
    "District Attorney",
    "Judge - Criminal District",
];

/// True if county may appear as a suffix (" X COUNTY") in the office title.
fn office_name_allows_county_suffix(office_name: Option<&str>) -> bool {
    let Some(n) = office_name else {
        return false;
    };
    let n_trimmed = n.trim();
    OFFICE_NAMES_ALLOWING_COUNTY_SUFFIX
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(n_trimmed))
}

/// Extract county from raw office title and return the title with county stripped.
/// Returns (county_if_any, office_title_without_county).
/// When office_name is in OFFICE_NAMES_ALLOWING_COUNTY_SUFFIX, tries " X COUNTY"
/// suffix first; otherwise tries "COUNTY_NAME - " prefix.
/// Extracted county is validated against the list of Texas counties.
pub fn extract_tx_county_from_office_title(
    input: &str,
    office_name: Option<&str>,
) -> (Option<String>, String) {
    if office_name_allows_county_suffix(office_name) {
        let (from_suffix, rest_after_suffix) = strip_county_suffix(input.trim());
        if let Some(county) = from_suffix {
            return (Some(normalize_county_name(&county)), rest_after_suffix);
        }
        return (None, normalize_whitespace(input.trim()));
    }
    let (from_prefix, rest) = strip_county_prefix(input);
    if let Some(ref county) = from_prefix {
        if is_valid_texas_county(county) {
            return (Some(normalize_county_name(county)), normalize_whitespace(rest));
        }
    }
    (None, normalize_whitespace(input.trim()))
}

/// Normalized office string for pattern matching: strip county prefix/suffix and lowercase.
fn normalized_office_part(input: &str) -> String {
    let (_, after_prefix) = strip_county_prefix(input);
    let (_, after_suffix) = strip_county_suffix(after_prefix);
    after_suffix.to_lowercase()
}

// ---------- Name & title ----------
// Order matters: more specific patterns should be checked first.

/// Converts party string to single letter for precinct chair: "Democratic" → "D", "Republican" → "R".
fn party_to_letter(party: Option<&str>) -> Option<&'static str> {
    match party.map(|p| p.trim()).filter(|p| !p.is_empty()) {
        Some(p) if p.eq_ignore_ascii_case("democratic") => Some("D"),
        Some(p) if p.eq_ignore_ascii_case("republican") => Some("R"),
        _ => None,
    }
}

pub fn extract_office_name(input: &str, party: Option<&str>) -> Option<String> {
    let input_lower = normalized_office_part(input);

    // Federal Offices
    if input_lower.contains("u.s. representative") || input_lower.contains("u. s. representative") {
        return Some("U.S. House".to_string());
    }
    if input_lower.contains("u.s. senator") || input_lower.contains("u. s. senator") {
        return Some("U.S. Senate".to_string());
    }

    // State Executive Offices
    if input_lower.contains("lieutenant governor") {
        return Some("Lieutenant Governor".to_string());
    }
    if input_lower.contains("governor") {
        return Some("Governor".to_string());
    }
    if input_lower.contains("attorney general") {
        return Some("Attorney General".to_string());
    }
    if input_lower.contains("state board of education") {
        return Some("State Board of Education".to_string());
    }
    if input_lower.contains("railroad commissioner") {
        return Some("Railroad Commissioner".to_string());
    }
    if input_lower.contains("commissioner of agriculture") {
        return Some("Commissioner of Agriculture".to_string());
    }
    if input_lower.contains("commissioner of the general land office") {
        return Some("Commissioner of the General Land Office".to_string());
    }
    if input_lower.contains("comptroller of public accounts") {
        return Some("Comptroller of Public Accounts".to_string());
    }

    // State Legislative Offices
    if input_lower.contains("state representative") {
        return Some("State House".to_string());
    }
    if input_lower.contains("state senator") {
        return Some("State Senate".to_string());
    }

    // State Judicial Offices
    if input_lower.contains("chief justice") && input_lower.contains("supreme court") {
        return Some("Chief Justice - Supreme Court".to_string());
    }
    if input_lower.contains("justice") && input_lower.contains("supreme court") && !input_lower.contains("chief") {
        return Some("Justice - Supreme Court".to_string());
    }
    if input_lower.contains("chief justice") && input_lower.contains("court of appeals") {
        return Some("Chief Justice - Court of Appeals".to_string());
    }
    if input_lower.contains("justice") && input_lower.contains("court of appeals") && !input_lower.contains("chief") {
        return Some("Justice - Court of Appeals".to_string());
    }
    if input_lower.contains("court of criminal appeals") {
        return Some("Judge - Court of Criminal Appeals".to_string());
    }
    if input_lower.contains("district judge") && input_lower.contains("judicial district") {
        return Some("Judge - District Court".to_string());
    }

    // District Attorney is for both State and County types
    if input_lower.contains("district attorney") {
        return Some("District Attorney".to_string());
    }

    // County Offices
    if input_lower.contains("county commissioner") {
        return Some("County Commissioner".to_string());
    }
    if input_lower.contains("county judge") {
        return Some("County Judge".to_string());
    }
    if input_lower.contains("county clerk") && input_lower.contains("district clerk") {
        return Some("County & District Clerk".to_string());
    }
    if input_lower.contains("county clerk") {
        return Some("County Clerk".to_string());
    }
    if input_lower.contains("district clerk") {
        return Some("District Clerk".to_string());
    }
    if input_lower.contains("county attorney") {
        return Some("County Attorney".to_string());
    }
    if input_lower.contains("county treasurer") {
        return Some("County Treasurer".to_string());
    }
    if input_lower.contains("county surveyor") {
        return Some("County Surveyor".to_string());
    }
    if input_lower.contains("county tax assessor-collector") {
        return Some("County Tax Assessor-Collector".to_string());
    }
    if input_lower.contains("county chair") {
        return Some("County Chair".to_string());
    }
    if input_lower.contains("justice of the peace") {
        return Some("Justice of the Peace".to_string());
    }
    if input_lower.contains("county constable") {
        return Some("County Constable".to_string());
    }
    if input_lower.contains("sheriff") {
        return Some("Sheriff".to_string());
    }

    // County Judicial Offices
    if input_lower.contains("1st multicounty court at law") {
        return Some("Judge - 1st Multicounty Court at Law".to_string());
    }
    if input_lower.contains("county civil court at law") {
        return Some("Judge - County Civil Court at Law".to_string());
    }
    if input_lower.contains("county criminal court of appeals") {
        return Some("Judge - County Criminal Court of Appeals".to_string());
    }
    if input_lower.contains("county criminal court") {
        return Some("Judge - County Criminal Court at Law".to_string());
    }
    if input_lower.contains("county probate court at law") {
        return Some("Judge - Probate Court".to_string());
    }
    if input_lower.contains("county court at law") {
        return Some("Judge - County Court at Law".to_string());
    }
    if input_lower.contains("criminal district judge") {
        return Some("Judge - Criminal District".to_string());
    }

    // VTD Precinct Chair: "Precinct Chair (D)" or "Precinct Chair (R)" when party is Democratic/Republican
    if input_lower.contains("precinct chair") {
        let name = party_to_letter(party)
            .map(|letter| format!("Precinct Chair ({})", letter))
            .unwrap_or_else(|| "Precinct Chair".to_string());
        return Some(name);
    }

    eprintln!("extract_office_name: no match for input: {:?}", input);
    None
}

/// Returns the display title for the office from its canonical name (from extract_office_name).
/// Only names with a different display title are matched; all others return the name unchanged.
pub fn extract_office_title(name: &str) -> Option<String> {
    let title = match name {
        "U.S. House" => "U.S. Representative",
        "U.S. Senate" => "U.S. Senator",
        "State House" => "State Representative",
        "State Senate" => "State Senator",
        "State Board of Education" => "State Board of Education Member",
        _ => return Some(name.to_string()),
    };
    Some(title.to_string())
}

/// Returns the chamber (Senate/House) from the canonical office name. Only U.S. Senate, U.S. House,
/// State Senate, and State House have a chamber; all others return None.
pub fn extract_office_chamber(name: &str) -> Option<db::Chamber> {
    match name {
        "U.S. Senate" | "State Senate" => Some(db::Chamber::Senate),
        "U.S. House" | "State House" => Some(db::Chamber::House),
        _ => None,
    }
}

// ---------- Scope (political_scope, election_scope, district_type) ----------

// Mappings follow populist-office-titles-map.csv. District Attorney is left blank;
// raw filing input will be used later to distinguish state judicial vs county.

/// Returns (political_scope, election_scope, district_type) for the given office name.
/// Takes raw filing input and optional county (e.g. for District Attorney: county set => local/county, else state/district/judicial).
/// Returns None when scope cannot be determined.
pub fn extract_office_scope(
    name: &str,
    county: Option<&str>,
) -> Option<(db::PoliticalScope, db::ElectionScope, Option<db::DistrictType>)> {
    use db::{DistrictType, ElectionScope, PoliticalScope};

    let out: (PoliticalScope, ElectionScope, Option<DistrictType>) = match name {
        "U.S. House" => (PoliticalScope::Federal, ElectionScope::District, Some(DistrictType::UsCongressional)),
        "U.S. Senate" => (PoliticalScope::Federal, ElectionScope::State, None),
        "Governor" | "Lieutenant Governor" | "Attorney General" | "Railroad Commissioner"
        | "Commissioner of Agriculture" | "Commissioner of the General Land Office" | "Comptroller of Public Accounts"
        | "Chief Justice - Supreme Court" | "Justice - Supreme Court" | "Judge - Court of Criminal Appeals" => {
            (PoliticalScope::State, ElectionScope::State, None)
        }
        "State Board of Education" => (PoliticalScope::State, ElectionScope::District, Some(DistrictType::BoardOfEducation)),
        "State House" => (PoliticalScope::State, ElectionScope::District, Some(DistrictType::StateHouse)),
        "State Senate" => (PoliticalScope::State, ElectionScope::District, Some(DistrictType::StateSenate)),
        "Chief Justice - Court of Appeals" | "Justice - Court of Appeals" => {
            (PoliticalScope::State, ElectionScope::District, Some(DistrictType::CourtOfAppeals))
        }
        "Judge - District Court" => (PoliticalScope::State, ElectionScope::District, Some(DistrictType::Judicial)),
        "District Attorney" => {
            if county.is_some() {
                (PoliticalScope::Local, ElectionScope::County, None)
            } else {
                (PoliticalScope::State, ElectionScope::District, Some(DistrictType::Judicial))
            }
        }
        "County Commissioner" => (PoliticalScope::Local, ElectionScope::District, Some(DistrictType::County)),
        "County Judge" | "County Clerk" | "District Clerk" | "County Attorney" | "County Treasurer"
        | "County Surveyor" | "County Tax Assessor-Collector" | "County Chair" | "County & District Clerk"
        | "Sheriff" | "Judge - County Court at Law" | "Judge - 1st Multicounty Court at Law" | "Judge - County Civil Court at Law"
        | "Judge - County Criminal Court of Appeals" | "Judge - County Criminal Court at Law"
        | "Judge - Probate Court" | "Judge - Criminal District" => (PoliticalScope::Local, ElectionScope::County, None),
        "Justice of the Peace" => (PoliticalScope::Local, ElectionScope::District, Some(DistrictType::JusticeOfThePeace)),
        "County Constable" => (PoliticalScope::Local, ElectionScope::District, Some(DistrictType::Constable)),
        "Precinct Chair" | "Precinct Chair (D)" | "Precinct Chair (R)" => {
            (PoliticalScope::Local, ElectionScope::District, Some(DistrictType::VotingPrecinct))
        }
        _ => return None,
    };
    Some(out)
}

// ---------- District ----------
// Note that these extractions work only because the seat is first stripped from the title.
// As more offices are ingested, will need to revisit this and add more patterns to the extractors.

/// Extracts all numbers from a string (e.g. "1, 5 & 6" or "2 & 6") and returns them in "X, X, X" format.
fn normalize_precinct_district_list(s: &str) -> String {
    let mut numbers = Vec::new();
    let mut current = String::new();
    for c in s.chars() {
        if c.is_ascii_digit() {
            current.push(c);
        } else {
            if !current.is_empty() {
                numbers.push(std::mem::take(&mut current));
            }
        }
    }
    if !current.is_empty() {
        numbers.push(current);
    }
    numbers.join(", ")
}

pub fn extract_office_district(input: &str) -> Option<String> {
    let input_lower = normalized_office_part(input);
    let extractors = TX_DISTRICT_EXTRACTORS.get_or_init(|| {
        vec![
            // [0] Generic "district N" – try last (broad; would false-match if run early)
            Regex::new(r"(?i)district\s+([0-9]{1,3}[A-Za-z]*)").unwrap(),
            // [1] Ordinal + "district" (e.g. 1st District) – must run after [3],[4] so we don't match inside "1st Court of Appeals District"
            Regex::new(r"(?i)([0-9]{1,3})(?:st|nd|rd|th)\s+district").unwrap(),
            // [2] Precinct list (e.g. "1, 2, 3", "2 & 6")
            Regex::new(r"(?i)precinct\s+([0-9][0-9\s,&]*)").unwrap(),
            // [3] Ordinal + "Court of Appeals District" – more specific than [1]
            Regex::new(r"(?i)([0-9]{1,3})(?:st|nd|rd|th)\s+court\s+of\s+appeals\s+district").unwrap(),
            // [4] Ordinal + "Judicial District" – more specific than [1]
            Regex::new(r"(?i)([0-9]{1,3})(?:st|nd|rd|th)\s+judicial\s+district").unwrap(),
            // [5] PCHR_<digits>_ (precinct chair)
            Regex::new(r"(?i)pchr_([0-9]+)_").unwrap(),
            // [6] "No." + number
            Regex::new(r"(?i)no\.\s*([0-9]{1,3}[A-Za-z]*)").unwrap(),
            // [7] "#" + number
            Regex::new(r"#\s*([0-9]{1,3}[A-Za-z]*)").unwrap(),
            // [8] "number" + number (e.g. COUNTY NUMBER 1)
            Regex::new(r"(?i)number\s+([0-9]{1,3}[A-Za-z]*)").unwrap(),
        ]
    });

    // Order: most specific first so broader patterns don't consume the wrong span.
    // 1. PCHR_ (unique to precinct chair)
    if input_lower.contains("pchr_") {
        if let Some(caps) = extractors[5].captures(input) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().to_string());
            }
        }
    }

    // 2. Court of Appeals / Judicial District (ordinal+district) before generic ordinal+district [1]
    if input_lower.contains("court of appeals") {
        if let Some(caps) = extractors[3].captures(input) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().to_string());
            }
        }
    }
    if input_lower.contains("judicial district") {
        if let Some(caps) = extractors[4].captures(input) {
            if let Some(m) = caps.get(1) {
                return Some(m.as_str().to_string());
            }
        }
    }

    // 3. Precinct list (guard: precinct but not precinct chair)
    if input_lower.contains("precinct") && !input_lower.contains("precinct chair") {
        if let Some(caps) = extractors[2].captures(input) {
            if let Some(m) = caps.get(1) {
                let s = m.as_str().trim();
                if !s.is_empty() {
                    return Some(normalize_precinct_district_list(s));
                }
            }
        }
    }

    // 4. Generic ordinal + "district" (e.g. 1st District, 83rd District)
    if let Some(caps) = extractors[1].captures(input) {
        if let Some(m) = caps.get(1) {
            return Some(m.as_str().to_string());
        }
    }

    // 5. Literal markers (#, No., NUMBER) – order doesn't overlap in practice
    if let Some(caps) = extractors[7].captures(input) {
        if let Some(m) = caps.get(1) {
            return Some(m.as_str().to_string());
        }
    }
    if let Some(caps) = extractors[6].captures(input) {
        if let Some(m) = caps.get(1) {
            return Some(m.as_str().to_string());
        }
    }
    if let Some(caps) = extractors[8].captures(input) {
        if let Some(m) = caps.get(1) {
            return Some(m.as_str().to_string());
        }
    }

    // 6. Generic "district N" last
    if let Some(caps) = extractors[0].captures(input) {
        if let Some(m) = caps.get(1) {
            return Some(m.as_str().to_string());
        }
    }

    None
}

// ---------- Seat ----------

/// Extracts the seat number after "place" (if present) and returns (seat, input_with_seat_stripped).
/// The stripped string has the "place N" segment removed and is trimmed, for use e.g. in district extraction.
pub fn extract_office_seat(input: &str) -> (Option<String>, String) {
    let re = TX_SEAT_EXTRACTORS.get_or_init(|| Regex::new(r"(?i)place\s+([0-9]{1,3})").unwrap());
    let stripped = re
        .replace(input, "")
        .trim()
        .trim_end_matches(',')
        .trim()
        .to_string();
    let seat = re
        .captures(input)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string());
    (seat, stripped)
}
