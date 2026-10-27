//! Texas office extractors: parse raw office title strings (e.g. from candidate filings)
//! into office_title, political_scope, election_scope, district_type, district, seat, county.
//!
//! Patterns aligned with populist-office-titles-map.csv and MN office extractors.

use std::sync::OnceLock;

use regex::Regex;

use crate::extractors::politician::title_case;

/// Strip "COUNTY_NAME - " prefix from raw title; returns (county_name_if_any, rest).
/// Only accepts the prefix when it matches a valid Texas county (TEXAS_COUNTIES), same as strip_county_suffix.
/// Example: "KLEBERG - COUNTY COMMISSIONER PRECINCT 4" -> (Some("Kleberg"), "COUNTY COMMISSIONER PRECINCT 4")
fn strip_county_prefix(input: &str) -> (Option<String>, &str) {
    let prefix_re = TX_COUNTY_PREFIX_REGEX
        .get_or_init(|| Regex::new(r"^\s*([A-Z][A-Za-z\s]+?)\s*-\s*(.*)$").unwrap());
    if let Some(caps) = prefix_re.captures(input) {
        if let (Some(county), Some(rest)) = (caps.get(1), caps.get(2)) {
            let county_str = county.as_str().trim();
            let rest_str = rest.as_str().trim();
            let county_norm = normalize_county_name(county_str);
            if !rest_str.is_empty() && county_str.len() >= 2 && is_valid_texas_county(&county_norm)
            {
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
    let mut counties: Vec<&str> = TEXAS_COUNTIES.to_vec();
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

static TX_COUNTY_PREFIX_REGEX: OnceLock<Regex> = OnceLock::new();
static TX_DISTRICT_EXTRACTORS: OnceLock<Vec<Regex>> = OnceLock::new();
static TX_SEAT_LABEL_REGEX: OnceLock<Regex> = OnceLock::new();
static TX_AT_LARGE_REGEX: OnceLock<Regex> = OnceLock::new();
static TX_SEAT_NUMBER_REGEX: OnceLock<Regex> = OnceLock::new();

/// All 254 Texas county names (title case), for validating extracted county strings.
const TEXAS_COUNTIES: [&str; 254] = [
    "Anderson",
    "Andrews",
    "Angelina",
    "Aransas",
    "Archer",
    "Armstrong",
    "Atascosa",
    "Austin",
    "Bailey",
    "Bandera",
    "Bastrop",
    "Baylor",
    "Bee",
    "Bell",
    "Bexar",
    "Blanco",
    "Borden",
    "Bosque",
    "Bowie",
    "Brazoria",
    "Brazos",
    "Brewster",
    "Briscoe",
    "Brooks",
    "Brown",
    "Burleson",
    "Burnet",
    "Caldwell",
    "Calhoun",
    "Callahan",
    "Cameron",
    "Camp",
    "Carson",
    "Cass",
    "Castro",
    "Chambers",
    "Cherokee",
    "Childress",
    "Clay",
    "Cochran",
    "Coke",
    "Coleman",
    "Collin",
    "Collingsworth",
    "Colorado",
    "Comal",
    "Comanche",
    "Concho",
    "Cooke",
    "Coryell",
    "Cottle",
    "Crane",
    "Crockett",
    "Crosby",
    "Culberson",
    "Dallam",
    "Dallas",
    "Dawson",
    "Deaf Smith",
    "Delta",
    "Denton",
    "DeWitt",
    "Dickens",
    "Dimmit",
    "Donley",
    "Duval",
    "Eastland",
    "Ector",
    "Edwards",
    "Ellis",
    "El Paso",
    "Erath",
    "Falls",
    "Fannin",
    "Fayette",
    "Fisher",
    "Floyd",
    "Foard",
    "Fort Bend",
    "Franklin",
    "Freestone",
    "Frio",
    "Gaines",
    "Galveston",
    "Garza",
    "Gillespie",
    "Glasscock",
    "Goliad",
    "Gonzales",
    "Gray",
    "Grayson",
    "Gregg",
    "Grimes",
    "Guadalupe",
    "Hale",
    "Hall",
    "Hamilton",
    "Hansford",
    "Hardeman",
    "Hardin",
    "Harris",
    "Harrison",
    "Hartley",
    "Haskell",
    "Hays",
    "Hemphill",
    "Henderson",
    "Hidalgo",
    "Hill",
    "Hockley",
    "Hood",
    "Hopkins",
    "Houston",
    "Howard",
    "Hudspeth",
    "Hunt",
    "Hutchinson",
    "Irion",
    "Jack",
    "Jackson",
    "Jasper",
    "Jeff Davis",
    "Jefferson",
    "Jim Hogg",
    "Jim Wells",
    "Johnson",
    "Jones",
    "Karnes",
    "Kaufman",
    "Kendall",
    "Kenedy",
    "Kent",
    "Kerr",
    "Kimble",
    "King",
    "Kinney",
    "Kleberg",
    "Knox",
    "La Salle",
    "Lamar",
    "Lamb",
    "Lampasas",
    "Lavaca",
    "Lee",
    "Leon",
    "Liberty",
    "Limestone",
    "Lipscomb",
    "Live Oak",
    "Llano",
    "Loving",
    "Lubbock",
    "Lynn",
    "Madison",
    "Marion",
    "Martin",
    "Mason",
    "Matagorda",
    "Maverick",
    "McCulloch",
    "McLennan",
    "McMullen",
    "Medina",
    "Menard",
    "Midland",
    "Milam",
    "Mills",
    "Mitchell",
    "Montague",
    "Montgomery",
    "Moore",
    "Morris",
    "Motley",
    "Nacogdoches",
    "Navarro",
    "Newton",
    "Nolan",
    "Nueces",
    "Ochiltree",
    "Oldham",
    "Orange",
    "Palo Pinto",
    "Panola",
    "Parker",
    "Parmer",
    "Pecos",
    "Polk",
    "Potter",
    "Presidio",
    "Rains",
    "Randall",
    "Reagan",
    "Real",
    "Red River",
    "Reeves",
    "Refugio",
    "Roberts",
    "Robertson",
    "Rockwall",
    "Runnels",
    "Rusk",
    "Sabine",
    "San Augustine",
    "San Jacinto",
    "San Patricio",
    "San Saba",
    "Schleicher",
    "Scurry",
    "Shackelford",
    "Shelby",
    "Sherman",
    "Smith",
    "Somervell",
    "Starr",
    "Stephens",
    "Sterling",
    "Stonewall",
    "Sutton",
    "Swisher",
    "Tarrant",
    "Taylor",
    "Terrell",
    "Terry",
    "Throckmorton",
    "Titus",
    "Tom Green",
    "Travis",
    "Trinity",
    "Tyler",
    "Upshur",
    "Upton",
    "Uvalde",
    "Val Verde",
    "Van Zandt",
    "Victoria",
    "Walker",
    "Waller",
    "Ward",
    "Washington",
    "Webb",
    "Wharton",
    "Wheeler",
    "Wichita",
    "Wilbarger",
    "Willacy",
    "Williamson",
    "Wilson",
    "Winkler",
    "Wise",
    "Wood",
    "Yoakum",
    "Young",
    "Zapata",
    "Zavala",
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
const OFFICE_NAMES_ALLOWING_COUNTY_SUFFIX: &[&str] =
    &["Criminal District Attorney", "Criminal District Judge"];

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
            let name = normalize_county_name(&county);
            return (Some(title_case(&name)), rest_after_suffix);
        }
        return (None, normalize_whitespace(input.trim()));
    }
    let (from_prefix, rest) = strip_county_prefix(input);
    if let Some(ref county) = from_prefix {
        if is_valid_texas_county(county) {
            let name = normalize_county_name(county);
            return (Some(title_case(&name)), normalize_whitespace(rest));
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

/// Special districts that *are* the district (do not extract a numeric district from the title).
/// Longer phrases first so "Water Control and Improvement District" wins over "Improvement District".
const SPECIAL_DISTRICT_NO_SUBDIVISIONS: &[&str] = &[
    "water control and improvement district",
    "municipal management district",
    "emergency services district",
    "fresh water supply district",
    "community library district",
    "municipal utility district",
    "special utility district",
    "the woodlands township",
    "improvement district",
    "navigation district",
];

/// Special districts with electoral subdivisions. Name is the text through the phrase.
/// `save_name_as_district` is true, and the local filing processor stores the subdivision
/// on `subdistrict`.
const SPECIAL_DISTRICTS: &[&str] = &[
    "groundwater conservation district",
    "authority",
    "community college",
];

fn first_phrase_match<'a>(input_lower: &str, phrases: &[&'a str]) -> Option<(usize, &'a str)> {
    for phrase in phrases {
        if let Some(pos) = input_lower.find(phrase) {
            return Some((pos, *phrase));
        }
    }
    None
}

/// Name from `… No. <word>`: through the end of that word, with "No." removed.
fn name_through_no_dot_word(input_lower: &str) -> Option<String> {
    let no_pos = input_lower.find("no.")?;
    let after_no = no_pos + 3;
    let rest = input_lower[after_no..].trim_start();
    let word_len = rest.find([' ', ',']).unwrap_or(rest.len());
    if word_len == 0 {
        return None;
    }
    let word_start = after_no + (input_lower[after_no..].len() - rest.len());
    let through = input_lower[..word_start + word_len].trim();
    let stripped = normalize_whitespace(&through.replace("no.", ""));
    if stripped.is_empty() {
        None
    } else {
        Some(stripped)
    }
}

pub fn is_special_district_with_subdivision(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    SPECIAL_DISTRICTS
        .iter()
        .any(|phrase| name_lower.contains(phrase))
}

fn title_case_district_name(s: &str) -> String {
    title_case(s).replace(" And ", " and ").replace(" Of ", " of ")
}

fn special_district_office_name(input_lower: &str) -> Option<(String, bool)> {
    if let Some((pos, phrase)) = first_phrase_match(input_lower, SPECIAL_DISTRICT_NO_SUBDIVISIONS) {
        if phrase == "the woodlands township" {
            return Some(("The Woodlands Township Board".to_string(), true));
        }
        let name = if input_lower.contains("no.") {
            name_through_no_dot_word(input_lower)
                .unwrap_or_else(|| input_lower[..pos + phrase.len()].trim().to_string())
        } else {
            input_lower[..pos + phrase.len()].trim().to_string()
        };
        if name.is_empty() {
            return None;
        }
        return Some((format!("{} Board", title_case_district_name(&name)), true));
    }

    if let Some((pos, phrase)) = first_phrase_match(input_lower, SPECIAL_DISTRICTS) {
        let through = input_lower[..pos + phrase.len()].trim();
        if !through.is_empty() {
            return Some((format!("{} Board", title_case_district_name(through)), true));
        }
    }

    None
}

fn named(name: impl Into<String>) -> Option<(String, bool)> {
    Some((name.into(), false))
}

/// Returns `(office_name, save_name_as_district)`.
/// `save_name_as_district` defaults to false, and the district is extracted from the title.
/// Special districts that are themselves the district (MUDs, ESDs, etc.) set it to true so
/// the office name is stored as `district`.
pub fn extract_office_name(input: &str, party: Option<&str>) -> Option<(String, bool)> {
    let input_lower = normalized_office_part(input);

    // Federal Offices
    if input_lower.contains("u.s. representative") || input_lower.contains("u. s. representative") {
        return named("U.S. House");
    }
    if input_lower.contains("u.s. senator") || input_lower.contains("u. s. senator") {
        return named("U.S. Senate");
    }

    // State Executive Offices
    if input_lower.contains("lieutenant governor") || input_lower.contains("lt. governor") {
        return named("Lieutenant Governor");
    }
    if input_lower.contains("governor") {
        return named("Governor");
    }
    if input_lower.contains("attorney general") {
        return named("Attorney General");
    }
    if input_lower.contains("state board of education") {
        return named("State Board of Education");
    }
    if input_lower.contains("railroad commissioner") {
        return named("Railroad Commissioner");
    }
    if input_lower.contains("commissioner of agriculture") {
        return named("Commissioner of Agriculture");
    }
    if input_lower.contains("commissioner of the general land office") {
        return named("Commissioner of the General Land Office");
    }
    if input_lower.contains("comptroller of public accounts") {
        return named("Comptroller of Public Accounts");
    }

    // State Legislative Offices
    if input_lower.contains("state representative") {
        return named("State House");
    }
    if input_lower.contains("state senator") {
        return named("State Senate");
    }

    // State Judicial Offices
    if input_lower.contains("chief justice") && input_lower.contains("supreme court") {
        return named("Chief Justice - Supreme Court");
    }
    if input_lower.contains("justice")
        && input_lower.contains("supreme court")
        && !input_lower.contains("chief")
    {
        return named("Justice - Supreme Court");
    }
    if input_lower.contains("chief justice") && input_lower.contains("court of appeals") {
        return named("Chief Justice - Court of Appeals");
    }
    if input_lower.contains("justice")
        && input_lower.contains("court of appeals")
        && !input_lower.contains("chief")
    {
        return named("Justice - Court of Appeals");
    }
    if input_lower.contains("court of criminal appeals") {
        return named("Judge - Court of Criminal Appeals");
    }
    if input_lower.contains("district judge") && input_lower.contains("judicial district") {
        return named("District Judge");
    }

    // District Attorney is for both State and County types
    if input_lower.contains("criminal district attorney") {
        return named("Criminal District Attorney");
    }
    if input_lower.contains("district attorney") {
        return named("District Attorney");
    }

    // County Offices
    if input_lower.contains("county commissioner") {
        return named("County Commissioner");
    }
    if input_lower.contains("county judge") {
        return named("County Judge");
    }
    if input_lower.contains("county and district clerk") {
        return named("County & District Clerk");
    }
    if input_lower.contains("county clerk") && input_lower.contains("district clerk") {
        return named("County & District Clerk");
    }
    if input_lower.contains("county clerk") {
        return named("County Clerk");
    }
    if input_lower.contains("district clerk") {
        return named("District Clerk");
    }
    if input_lower.contains("county attorney") {
        return named("County Attorney");
    }
    if input_lower.contains("county treasurer") {
        return named("County Treasurer");
    }
    if input_lower.contains("county surveyor") {
        return named("County Surveyor");
    }
    if input_lower.contains("sheriff") && input_lower.contains("county tax assessor-collector") {
        return named("Sheriff & County Tax Assessor-Collector");
    }
    if input_lower.contains("county tax assessor-collector") {
        return named("County Tax Assessor-Collector");
    }
    if input_lower.contains("county chair") {
        let name = party_to_letter(party)
            .map(|letter| format!("County Chair ({})", letter))
            .unwrap_or_else(|| "County Chair".to_string());
        return named(name);
    }
    if input_lower.contains("justice of the peace") {
        return named("Justice of the Peace");
    }
    if input_lower.contains("county constable") {
        return named("County Constable");
    }
    if input_lower.contains("sheriff") {
        return named("Sheriff");
    }
    if input_lower.contains("department of education") {
        return named("County School Trustee");
    }

    // County Judicial Offices
    if input_lower.contains("1st multicounty court at law") {
        return named("Judge - 1st Multicounty Court at Law");
    }
    if input_lower.contains("county civil court at law") {
        return named("Judge - County Civil Court at Law");
    }
    if input_lower.contains("county criminal court of appeals") {
        return named("Judge - County Criminal Court of Appeals");
    }
    if input_lower.contains("county criminal court") {
        return named("Judge - County Criminal Court at Law");
    }
    if input_lower.contains("probate court") {
        return named("Judge - Probate Court");
    }
    if input_lower.contains("county court at law") {
        return named("Judge - County Court at Law");
    }
    if input_lower.contains("criminal district judge") {
        return named("Criminal District Judge");
    }

    // Local Offices
    if input_lower.contains("independent school district") {
        return named("School Board");
    }
    
    if input_lower.contains("hospital district") {
        return named("Hospital District Board");
    }
    
    if let Some(special) = special_district_office_name(&input_lower) {
        return Some(special);
    }
    if input_lower.contains("city council") || input_lower.contains("councilmember") || input_lower.contains("council member") || input_lower.contains("council") {
        return named("City Council");
    }
    if input_lower.contains("mayor") {
        return named("Mayor");
    }
    if input_lower.contains("city commissioner") {
        return named("City Commissioner");
    }
    if input_lower.contains("alderman") {
        return named("Alderman");
    }
    
    if let Some(pos) = input_lower.find("appraisal district") {
        let through = input_lower[..pos + "appraisal district".len()]
            .trim()
            .trim_end_matches([' ', ',', '-', ';', ':'])
            .trim();
        if !through.is_empty() {
            return named(format!("{} Board", title_case(through)));
        }
    }

    // VTD Precinct Chair: "Precinct Chair (D)" or "Precinct Chair (R)" when party is Democratic/Republican
    if input_lower.contains("precinct chair") {
        let name = party_to_letter(party)
            .map(|letter| format!("Precinct Chair ({})", letter))
            .unwrap_or_else(|| "Precinct Chair".to_string());
        return named(name);
    }

    eprintln!("extract_office_name: no match for input: {:?}", input);
    None
}

/// Returns the display title for the office from its canonical name (from extract_office_name).
/// Only names with a different display title are matched; all others return the name unchanged.
pub fn extract_office_title(name: &str) -> Option<String> {
    if name.ends_with("Board") {
        return Some(format!("{} Member", name));
    }
    let title = match name {
        "U.S. House" => "U.S. Representative",
        "U.S. Senate" => "U.S. Senator",
        "State House" => "State Representative",
        "State Senate" => "State Senator",
        "State Board of Education" => "State Board of Education Member",
        "City Council" => "City Council Member",
        "Town Council" => "Town Council Member",
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
/// Takes raw filing input, optional county, optional seat (e.g. for Department of Education + Harris), and optional district.
/// Returns None when scope cannot be determined.
pub fn extract_office_scope(
    name: &str,
    county: Option<&str>,
    seat: Option<&str>,
    district: Option<&str>,
) -> Option<(
    db::PoliticalScope,
    db::ElectionScope,
    Option<db::DistrictType>,
)> {
    use db::{DistrictType, ElectionScope, PoliticalScope};

    let out: (PoliticalScope, ElectionScope, Option<DistrictType>) = match name {
        // County School Trustee Positions 3, 5, and 7 are At-Large
        // Position 1 is District 2
        // Position 2 is District 4
        // Position 4 is District 3
        // Position 6 is District 1
        "County School Trustee" => {
            let is_harris = county
                .map(|c| {
                    let t = c.trim();
                    t.eq_ignore_ascii_case("Harris")
                })
                .unwrap_or(false);
            if is_harris {
                match seat.and_then(seat_number) {
                    Some("7") | Some("5") | Some("3") => {
                        (PoliticalScope::Local, ElectionScope::County, None)
                    }
                    _ => (
                        PoliticalScope::Local,
                        ElectionScope::District,
                        Some(DistrictType::County),
                    ),
                }
            } else {
                (PoliticalScope::Local, ElectionScope::County, None)
            }
        }
        "U.S. House" => (
            PoliticalScope::Federal,
            ElectionScope::District,
            Some(DistrictType::UsCongressional),
        ),
        "U.S. Senate" => (PoliticalScope::Federal, ElectionScope::State, None),
        "Governor"
        | "Lieutenant Governor"
        | "Attorney General"
        | "Railroad Commissioner"
        | "Commissioner of Agriculture"
        | "Commissioner of the General Land Office"
        | "Comptroller of Public Accounts"
        | "Chief Justice - Supreme Court"
        | "Justice - Supreme Court"
        | "Judge - Court of Criminal Appeals" => {
            (PoliticalScope::State, ElectionScope::State, None)
        }
        "State Board of Education" => (
            PoliticalScope::State,
            ElectionScope::District,
            Some(DistrictType::BoardOfEducation),
        ),
        "State House" => (
            PoliticalScope::State,
            ElectionScope::District,
            Some(DistrictType::StateHouse),
        ),
        "State Senate" => (
            PoliticalScope::State,
            ElectionScope::District,
            Some(DistrictType::StateSenate),
        ),
        "Chief Justice - Court of Appeals" | "Justice - Court of Appeals" => {
            if district == Some("15") {
                (PoliticalScope::State, ElectionScope::State, None)
            } else {
                (
                    PoliticalScope::State,
                    ElectionScope::District,
                    Some(DistrictType::CourtOfAppeals),
                )
            }
        }
        "District Judge" => (
            PoliticalScope::State,
            ElectionScope::District,
            Some(DistrictType::Judicial),
        ),
        "District Attorney" => (
            PoliticalScope::State,
            ElectionScope::District,
            Some(DistrictType::Judicial),
        ),
        "Criminal District Attorney" => (PoliticalScope::Local, ElectionScope::County, None),

        "County Commissioner" => (
            PoliticalScope::Local,
            ElectionScope::District,
            Some(DistrictType::County),
        ),
        "County Judge"
        | "County Clerk"
        | "District Clerk"
        | "County Attorney"
        | "County Treasurer"
        | "County Surveyor"
        | "County Tax Assessor-Collector"
        | "County Chair (D)"
        | "County Chair (R)"
        | "County & District Clerk"
        | "Sheriff"
        | "Judge - County Court at Law"
        | "Judge - 1st Multicounty Court at Law"
        | "Judge - County Civil Court at Law"
        | "Judge - County Criminal Court of Appeals"
        | "Judge - County Criminal Court at Law"
        | "Judge - Probate Court"
        | "Criminal District Judge" => (PoliticalScope::Local, ElectionScope::County, None),
        "Justice of the Peace" => {
            if district.map(|d| !d.trim().is_empty()).unwrap_or(false) {
                (
                    PoliticalScope::Local,
                    ElectionScope::District,
                    Some(DistrictType::JusticeOfThePeace),
                )
            } else {
                (PoliticalScope::Local, ElectionScope::County, None)
            }
        }
        "County Constable" => {
            if district.map(|d| !d.trim().is_empty()).unwrap_or(false) {
                (
                    PoliticalScope::Local,
                    ElectionScope::District,
                    Some(DistrictType::Constable),
                )
            } else {
                (PoliticalScope::Local, ElectionScope::County, None)
            }
        }
        "Precinct Chair" | "Precinct Chair (D)" | "Precinct Chair (R)" => (
            PoliticalScope::Local,
            ElectionScope::District,
            Some(DistrictType::VotingPrecinct),
        ),
        "Mayor" | "City Commissioner" => (PoliticalScope::Local, ElectionScope::City, None),
        "City Council" | "Alderman" => {
            if district.map(|d| !d.trim().is_empty()).unwrap_or(false) {
                (
                    PoliticalScope::Local,
                    ElectionScope::District,
                    Some(DistrictType::City),
                )
            } else {
                (PoliticalScope::Local, ElectionScope::City, None)
            }
        }
        "School Board" => (
            PoliticalScope::Local,
            ElectionScope::District,
            Some(DistrictType::School),
        ),
        "Hospital District Board" => (
            if district.map(|d| !d.trim().is_empty()).unwrap_or(false) {
                (
                    PoliticalScope::Local,
                    ElectionScope::District,
                    Some(DistrictType::County),
                )
            } else {
                (PoliticalScope::Local, ElectionScope::County, None)
            }
        ),
        name if name.ends_with("Appraisal District Board") => {
            (PoliticalScope::Local, ElectionScope::County, None)
        }
        name if name.ends_with("College Board") => (
            PoliticalScope::Local,
            ElectionScope::District,
            Some(DistrictType::TxCollege),
        ),
        name if is_other_special_district_board(name) => (
            PoliticalScope::Local,
            ElectionScope::District,
            Some(DistrictType::TxSpecial),
        ),
        _ => return None,
    };
    Some(out)
}

fn is_other_special_district_board(name: &str) -> bool {
    let lower = name.to_lowercase();
    if !lower.ends_with(" board") {
        return false;
    }
    SPECIAL_DISTRICT_NO_SUBDIVISIONS
        .iter()
        .chain(["groundwater conservation district", "authority"].iter())
        .any(|phrase| lower.contains(phrase))
}

// ---------- District ----------
// Note that these extractions work only because the seat is first stripped from the title.
// As more offices are ingested, will need to revisit this and add more patterns to the extractors.

/// Extracts all numbers from a string (e.g. "1, 5 & 6" or "2 & 6") and returns them in "X, X, X" format.
fn normalize_precinct_district_list(s: &str) -> String {
    let mut values = Vec::new();
    let mut current = String::new();
    for c in s.chars() {
        if c == '&' || c == ',' || c.is_ascii_whitespace() {
            if !current.is_empty() {
                values.push(std::mem::take(&mut current));
            }
        } else {
            current.push(c);
        }
    }
    if !current.is_empty() {
        values.push(current);
    }
    values.join(", ")
}

/// For PCHR segment: ignore leading letters and "."/space; ignore "." throughout; return numbers + any letters after.
fn normalize_precinct_district(segment: &str) -> Option<String> {
    let trimmed =
        segment.trim_start_matches(|c: char| c.is_alphabetic() || c == '.' || c.is_whitespace());
    let without_dots = trimmed.replace('.', "");
    let out = without_dots.trim();
    if out.is_empty() {
        None
    } else {
        Some(out.to_string())
    }
}

pub fn extract_office_district(
    input: &str,
    seat: Option<&str>,
    municipality: Option<&str>,
    office_name: Option<&str>,
) -> Option<String> {
    if let Some(district) = district_from_office_exceptions(seat, municipality, office_name) {
        return Some(district);
    }

    let input_lower = normalized_office_part(input);
    let extractors = TX_DISTRICT_EXTRACTORS.get_or_init(|| {
        vec![
            // [0] Generic "district N" or a single-letter district ("District B"). Tried last.
            // The letter must be a whole word so "District Board" and "District Attorney" do not match.
            Regex::new(r"(?i)district\s+([0-9]{1,3}[A-Za-z]*|[A-Za-z])\b").unwrap(),
            // [1] Ordinal + "district" (e.g. 1st District) – must run after [3],[4] so we don't match inside "1st Court of Appeals District"
            Regex::new(r"(?i)([0-9]{1,3})(?:st|nd|rd|th)\s+district").unwrap(),
            // [2] Precinct list (e.g. "1, 2, 3", "2 & 6") or Pct N (e.g. "Pct 2")
            Regex::new(r"(?i)(?:precinct|pct\.?)\s+([0-9][0-9\s,&]*)").unwrap(),
            // [3] Ordinal + "Court of Appeals District" – more specific than [1]
            Regex::new(r"(?i)([0-9]{1,3})(?:st|nd|rd|th)\s+court\s+of\s+appeals\s+district")
                .unwrap(),
            // [4] Ordinal + "Judicial District" – more specific than [1]
            Regex::new(r"(?i)([0-9]{1,3})(?:st|nd|rd|th)\s+judicial\s+district").unwrap(),
            // [5] PCHR_<segment>_rep|_dem (precinct chair): capture segment between pchr_ and _rep/_dem; then ignore leading letters, ignore "." throughout, keep numbers + letters after
            Regex::new(r"(?i)pchr_(.+?)_(?:rep|dem)").unwrap(),
            // [6] "No." + number/label or "N & M" list only (no comma: "No. 1, Place 2" must not capture "1, place 2")
            Regex::new(r"(?i)no\.\s*([0-9][0-9A-Za-z\s&\-]*)").unwrap(),
            // [7] "#" + number/label
            Regex::new(r"#\s*([0-9][0-9A-Za-z-]*)").unwrap(),
            // [8] "number" + number/label (e.g. COUNTY NUMBER 1)
            Regex::new(r"(?i)number\s+([0-9][0-9A-Za-z-]*)").unwrap(),
            // [9] "Precinct Chair" + number/list/alpha (e.g. "Precinct Chair 324", "Precinct Chair, Pct 2", "Precinct Chair - 4B", "Precinct Chair, Precinct 4056"); capture is non-greedy: digits + optional letter, or list (N, M / N & M) so we don't consume trailing "precinct N"
            Regex::new(r"(?i)precinct\s+chair\s*[,]?\s*(?:\s*-\s*)?\s*(?:pct\.?\s*)?(?:precinct\s+)?\s*([0-9]+[A-Za-z]?(?:\s*[,&]\s*[0-9]+[A-Za-z]?)*)").unwrap(),
        ]
    });

    // Order: most specific first so broader patterns don't consume the wrong span.
    // 1. PCHR_ (unique to precinct chair)
    if input_lower.contains("pchr_") {
        if let Some(caps) = extractors[5].captures(input) {
            if let Some(m) = caps.get(1) {
                if let Some(normalized) = normalize_precinct_district(m.as_str()) {
                    return Some(normalized);
                }
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
    if (input_lower.contains("precinct") || input_lower.contains("pct"))
        && !input_lower.contains("precinct chair")
    {
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
            let s = m.as_str().trim();
            if !s.is_empty() {
                if s.contains('&') {
                    return Some(normalize_precinct_district_list(s));
                }
                return Some(s.to_string());
            }
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

    // 7. "Precinct Chair" + number/list/alpha (e.g. "Precinct Chair 324", "Precinct Chair, Pct 2", "Precinct Chair - 4B", "Precinct Chair, Precinct 4056"); [9] matches directly
    if input_lower.contains("precinct chairman") {
        if let Some(caps) = extractors[2].captures(input) {
            if let Some(m) = caps.get(1) {
                let s = m.as_str().trim();
                if !s.is_empty() {
                    return Some(normalize_precinct_district_list(s));
                }
            }
        }
        return None;
    }

    if input_lower.contains("precinct chair") {
        if let Some(caps) = extractors[9].captures(input) {
            if let Some(m) = caps.get(1) {
                let s = m.as_str().trim();
                if !s.is_empty() {
                    if s.contains('&') || s.contains(',') {
                        return Some(normalize_precinct_district_list(s));
                    }
                    return Some(s.to_string());
                }
            }
        }
        return None;
    }

    // 8. Harris County Department of Education: district depends on seat (place) number
    if input_lower.contains("harris county department of education") {
        let seat_trimmed = seat.map(|s| s.trim()).filter(|s| !s.is_empty());
        if let Some(s) = seat_trimmed {
            return Some(match seat_number(s).unwrap_or(s) {
                "3" | "5" | "7" => "At Large".to_string(),
                "1" => "2".to_string(),
                "2" => "4".to_string(),
                "4" => "3".to_string(),
                "6" => "1".to_string(),
                _ => return None,
            });
        }
    }

    None
}

// ---------- Seat ----------

/// Numeric id from a seat string (`"Place 1"`, `"Position No. 4"`, `"3"`).
fn seat_number(seat: &str) -> Option<&str> {
    let re = TX_SEAT_NUMBER_REGEX.get_or_init(|| {
        Regex::new(r"(?i)(?:place|seat|position)\s+(?:no\.?\s*)?([0-9]{1,3})").unwrap()
    });
    if let Some(caps) = re.captures(seat) {
        return caps.get(1).map(|m| m.as_str());
    }
    let trimmed = seat.trim();
    if !trimmed.is_empty() && trimmed.chars().all(|c| c.is_ascii_digit()) {
        Some(trimmed)
    } else {
        None
    }
}

fn names_eq(value: Option<&str>, expected: &str) -> bool {
    value.is_some_and(|v| v.trim().eq_ignore_ascii_case(expected))
}

fn district_from_office_exceptions(
    seat: Option<&str>,
    municipality: Option<&str>,
    office_name: Option<&str>,
) -> Option<String> {
    let n = seat.and_then(seat_number)?.parse::<u32>().ok()?;
    if names_eq(municipality, "The Colony") && names_eq(office_name, "City Council") {
        if (3..=6).contains(&n) {
            return Some(n.to_string());
        }
        return None;
    }
    if names_eq(office_name, "Fort Bend School Board") {
        return match n {
            1..=3 => Some("West".to_string()),
            5..=7 => Some("East".to_string()),
            _ => None,
        };
    }
    None
}

fn ensure_at_large_suffix(seat: &str) -> String {
    let lower = seat.to_ascii_lowercase();
    if lower.contains("at-large") || lower.contains("at large") {
        seat.to_string()
    } else {
        format!("{} - At-Large", seat)
    }
}

fn apply_seat_exceptions(
    seat: Option<String>,
    municipality: Option<&str>,
    office_name: Option<&str>,
) -> Option<String> {
    let seat = seat?;
    let n = seat_number(&seat);
    if names_eq(municipality, "The Colony")
        && names_eq(office_name, "City Council")
        && matches!(n, Some("1") | Some("2"))
    {
        return Some(ensure_at_large_suffix(&seat));
    }
    if names_eq(office_name, "Fort Bend School Board") && n == Some("4") {
        return Some(ensure_at_large_suffix(&seat));
    }
    Some(seat)
}

fn normalize_seat_label(label: &str) -> &'static str {
    match label.to_ascii_lowercase().as_str() {
        "place" => "Place",
        "seat" => "Seat",
        "position" => "Position",
        _ => "Place",
    }
}

fn strip_seat_artifacts(s: &str) -> String {
    static MULTI_COMMA: OnceLock<Regex> = OnceLock::new();
    let re = MULTI_COMMA.get_or_init(|| Regex::new(r",\s*,+").unwrap());
    let collapsed = re.replace_all(s, ",");
    normalize_whitespace(&collapsed)
        .trim_matches(|c: char| c.is_whitespace() || matches!(c, ',' | ';' | ':'))
        .trim_matches(|c: char| c.is_whitespace() || matches!(c, '-' | '–' | '—'))
        .trim()
        .to_string()
}

/// Extracts a labeled seat (`Place`, `Seat`, or `Position`) and optional At-Large suffix.
///
/// `num_elect` controls At-Large: if it is 2 or greater, At-Large is stripped and ignored
/// (the race elects multiple at-large winners). Otherwise At-Large is appended, e.g.
/// `"Place 1 - At-Large"`.
///
/// Optional `"No."` between the label and number is discarded (`"Place No. 1"` → `"Place 1"`).
/// `municipality` and `office_name` apply office-specific At-Large rules (The Colony City
/// Council seats 1–2, Fort Bend School Board seat 4).
/// Returns `(seat, input_with_seat_stripped)` so district extraction can run on the remainder.
pub fn extract_office_seat(
    input: &str,
    num_elect: Option<i32>,
    municipality: Option<&str>,
    office_name: Option<&str>,
) -> (Option<String>, String) {
    let labeled_re = TX_SEAT_LABEL_REGEX.get_or_init(|| {
        Regex::new(r"(?i)\b(?P<label>place|seat|position)\s+(?:no\.?\s*)?(?P<number>[0-9]{1,3})\b")
            .unwrap()
    });
    let at_large_re =
        TX_AT_LARGE_REGEX.get_or_init(|| Regex::new(r"(?i)\bat[\s\-–—]+large\b").unwrap());

    let labeled_caps = labeled_re.captures(input);
    let labeled_seat = labeled_caps.as_ref().map(|caps| {
        let label = normalize_seat_label(caps.name("label").map(|m| m.as_str()).unwrap_or("place"));
        let number = caps.name("number").map(|m| m.as_str()).unwrap_or("");
        format!("{} {}", label, number)
    });
    let at_large_match = at_large_re.find(input);
    let ignore_at_large = num_elect.is_some_and(|n| n >= 2);

    let seat = match (
        labeled_seat.as_deref(),
        at_large_match.is_some(),
        ignore_at_large,
    ) {
        (Some(labeled), true, false) => Some(format!("{} - At-Large", labeled)),
        (Some(labeled), _, _) => Some(labeled.to_string()),
        (None, true, false) => Some("At-Large".to_string()),
        (None, _, _) => None,
    };
    let seat = apply_seat_exceptions(seat, municipality, office_name);

    let mut ranges: Vec<std::ops::Range<usize>> = Vec::new();
    if let Some(caps) = labeled_caps.as_ref() {
        if let Some(full) = caps.get(0) {
            ranges.push(full.range());
        }
    }
    if let Some(m) = at_large_match {
        ranges.push(m.range());
    }
    ranges.sort_by_key(|r| std::cmp::Reverse(r.start));
    let mut stripped = input.to_string();
    for range in ranges {
        stripped.replace_range(range, " ");
    }

    (seat, strip_seat_artifacts(&stripped))
}

#[cfg(test)]
mod extract_office_scope_tests {
    use super::extract_office_scope;
    use db::{DistrictType, ElectionScope, PoliticalScope};

    fn scope(
        name: &str,
        district: Option<&str>,
    ) -> Option<(PoliticalScope, ElectionScope, Option<DistrictType>)> {
        extract_office_scope(name, None, None, district)
    }

    #[test]
    fn local_city_offices() {
        assert_eq!(
            scope("Mayor", None),
            Some((PoliticalScope::Local, ElectionScope::City, None))
        );
        assert_eq!(
            scope("City Commissioner", None),
            Some((PoliticalScope::Local, ElectionScope::City, None))
        );
        assert_eq!(
            scope("City Council", None),
            Some((PoliticalScope::Local, ElectionScope::City, None))
        );
        assert_eq!(
            scope("Alderman", Some("2")),
            Some((
                PoliticalScope::Local,
                ElectionScope::District,
                Some(DistrictType::City)
            ))
        );
    }

    #[test]
    fn local_boards() {
        assert_eq!(
            scope("Somerset School Board", None),
            Some((
                PoliticalScope::Local,
                ElectionScope::District,
                Some(DistrictType::School)
            ))
        );
        assert_eq!(
            scope("Fort Bend Central Appraisal District Board", None),
            Some((PoliticalScope::Local, ElectionScope::County, None))
        );
        assert_eq!(
            scope("Austin Community College Board", None),
            Some((
                PoliticalScope::Local,
                ElectionScope::District,
                Some(DistrictType::TxCollege)
            ))
        );
        assert_eq!(
            scope("Harris County Hospital District Board", None),
            Some((PoliticalScope::Local, ElectionScope::County, None))
        );
        assert_eq!(
            scope("Harris County Hospital District Board", Some("1")),
            Some((
                PoliticalScope::Local,
                ElectionScope::District,
                Some(DistrictType::County)
            ))
        );
        assert_eq!(
            scope("The Woodlands Township Board", None),
            Some((
                PoliticalScope::Local,
                ElectionScope::District,
                Some(DistrictType::TxSpecial)
            ))
        );
        assert_eq!(
            scope("Fort Bend County Municipal Utility District 50 Board", None),
            Some((
                PoliticalScope::Local,
                ElectionScope::District,
                Some(DistrictType::TxSpecial)
            ))
        );
    }
}

#[cfg(test)]
mod extract_office_name_tests {
    use super::{extract_office_name, extract_office_title};

    fn name_of(input: &str) -> Option<String> {
        extract_office_name(input, None).map(|(n, _)| n)
    }

    #[test]
    fn independent_school_district_becomes_school_board() {
        assert_eq!(
            extract_office_name("Somerset Independent School District, District 3", None),
            Some(("Somerset School Board".to_string(), true))
        );
        assert_eq!(
            extract_office_name("Santa Fe Independent School District, Position 5", None),
            Some(("Santa Fe School Board".to_string(), true))
        );
        assert_eq!(
            extract_office_name("Reagan County Independent School District, District 1", None),
            Some(("Reagan County School Board".to_string(), true))
        );
    }

    #[test]
    fn appraisal_district_becomes_appraisal_district_board() {
        assert_eq!(
            name_of("Fort Bend Central Appraisal District, At-Large Position No. 1").as_deref(),
            Some("Fort Bend Central Appraisal District Board")
        );
        assert_eq!(
            name_of("Collin County Appraisal District, Place 3").as_deref(),
            Some("Collin County Appraisal District Board")
        );
        assert_eq!(
            name_of("Harris Central Appraisal District, Board of Directors, At-Large, Place 2")
                .as_deref(),
            Some("Harris Central Appraisal District Board")
        );
    }

    #[test]
    fn special_districts_with_subdivisions_include_matched_phrase() {
        assert_eq!(
            extract_office_name(
                "Hays Trinity Groundwater Conservation District Board Director for Single Member District #2",
                None
            ),
            Some(("Hays Trinity Groundwater Conservation District Board".to_string(), true))
        );
        assert_eq!(
            extract_office_name("Harris County Hospital District, District 1", None),
            Some(("Harris County Hospital District Board".to_string(), false))
        );
        assert_eq!(
            extract_office_name(
                "Medina County Hospital District, Board of Directors, At-Large, Place 5",
                None
            ),
            Some(("Medina County Hospital District Board".to_string(), false))
        );
    }

    #[test]
    fn special_district_names_skip_district_extraction() {
        assert_eq!(
            extract_office_name(
                "Fort Bend County Municipal Utility District No. 50 Position 3",
                None
            ),
            Some((
                "Fort Bend County Municipal Utility District 50 Board".to_string(),
                true
            ))
        );
        assert_eq!(
            extract_office_name("Harris County Emergency Services District No. 12, Place 1", None),
            Some((
                "Harris County Emergency Services District 12 Board".to_string(),
                true
            ))
        );
        assert_eq!(
            extract_office_name("Collin County Municipal Utility District Place 1", None),
            Some((
                "Collin County Municipal Utility District Board".to_string(),
                true
            ))
        );
        assert_eq!(
            extract_office_name("The Woodlands Township Position 2", None),
            Some(("The Woodlands Township Board".to_string(), true))
        );
        assert_eq!(
            extract_office_name(
                "Harris County Water Control and Improvement District No. 1",
                None
            ),
            Some((
                "Harris County Water Control and Improvement District 1 Board".to_string(),
                true
            ))
        );
    }

    #[test]
    fn board_title_appends_member() {
        assert_eq!(
            extract_office_title("Somerset School Board").as_deref(),
            Some("Somerset School Board Member")
        );
        assert_eq!(
            extract_office_title("Fort Bend School Board").as_deref(),
            Some("Fort Bend School Board Member")
        );
        assert_eq!(
            extract_office_title("Fort Bend County Municipal Utility District 50 Board")
                .as_deref(),
            Some("Fort Bend County Municipal Utility District 50 Board Member")
        );
    }
}

#[cfg(test)]
mod extract_office_seat_tests {
    use super::{extract_office_district, extract_office_seat};

    #[test]
    fn single_letter_district_is_extracted() {
        assert_eq!(
            extract_office_district(
                "City of Buda, City Council, Single Member District B",
                None,
                Some("Buda"),
                Some("City Council"),
            ),
            Some("B".to_string())
        );
        assert_eq!(
            extract_office_district("Judge, 83rd District", None, None, None),
            Some("83".to_string())
        );
        assert_eq!(
            extract_office_district("District 4B", None, None, None),
            Some("4B".to_string())
        );
        assert_eq!(
            extract_office_district("District Attorney", None, None, None),
            None
        );
        assert_eq!(
            extract_office_district("Harris County Hospital District Board", None, None, None),
            None
        );
    }

    #[test]
    fn position_is_a_seat_not_a_district() {
        let input = "Fort Bend Independent School District, Position 4";
        let (seat, stripped) = extract_office_seat(input, None, None, None);
        assert_eq!(seat.as_deref(), Some("Position 4"));
        assert_eq!(stripped, "Fort Bend Independent School District");
        assert_eq!(
            extract_office_district(&stripped, seat.as_deref(), None, None),
            None
        );
    }

    #[test]
    fn place_seat_and_no_dot() {
        let (seat, stripped) = extract_office_seat("City Council, Place No. 1", None, None, None);
        assert_eq!(seat.as_deref(), Some("Place 1"));
        assert_eq!(stripped, "City Council");

        let (seat, stripped) = extract_office_seat("City Council Seat 2", None, None, None);
        assert_eq!(seat.as_deref(), Some("Seat 2"));
        assert_eq!(stripped, "City Council");
    }

    #[test]
    fn at_large_appended_unless_multi_elect() {
        let (seat, stripped) =
            extract_office_seat("City Council Place 1 At-Large", None, None, None);
        assert_eq!(seat.as_deref(), Some("Place 1 - At-Large"));
        assert_eq!(stripped, "City Council");

        let (seat, stripped) =
            extract_office_seat("City Council Place 1 At-Large", Some(3), None, None);
        assert_eq!(seat.as_deref(), Some("Place 1"));
        assert_eq!(stripped, "City Council");

        let (seat, stripped) = extract_office_seat("City Council At-Large", None, None, None);
        assert_eq!(seat.as_deref(), Some("At-Large"));
        assert_eq!(stripped, "City Council");

        let (seat, stripped) = extract_office_seat("City Council At-Large", Some(2), None, None);
        assert_eq!(seat, None);
        assert_eq!(stripped, "City Council");
    }

    #[test]
    fn the_colony_city_council_seat_and_district() {
        let muni = Some("The Colony");
        let office = Some("City Council");

        let (seat, _) = extract_office_seat("City Council Place 1", None, muni, office);
        assert_eq!(seat.as_deref(), Some("Place 1 - At-Large"));
        assert_eq!(
            extract_office_district("City Council", seat.as_deref(), muni, office),
            None
        );

        let (seat, _) = extract_office_seat("City Council Place 2", None, muni, office);
        assert_eq!(seat.as_deref(), Some("Place 2 - At-Large"));

        let (seat, _) = extract_office_seat("City Council Place 3", None, muni, office);
        assert_eq!(seat.as_deref(), Some("Place 3"));
        assert_eq!(
            extract_office_district("City Council", seat.as_deref(), muni, office),
            Some("3".to_string())
        );

        let (seat, _) = extract_office_seat("City Council Place 6", None, muni, office);
        assert_eq!(seat.as_deref(), Some("Place 6"));
        assert_eq!(
            extract_office_district("City Council", seat.as_deref(), muni, office),
            Some("6".to_string())
        );
    }

    #[test]
    fn fort_bend_school_board_seat_and_district() {
        let office = Some("Fort Bend School Board");

        let (seat, _) = extract_office_seat("Fort Bend School Board Position 2", None, None, office);
        assert_eq!(seat.as_deref(), Some("Position 2"));
        assert_eq!(
            extract_office_district("Fort Bend School Board", seat.as_deref(), None, office),
            Some("West".to_string())
        );

        let (seat, _) = extract_office_seat("Fort Bend School Board Position 4", None, None, office);
        assert_eq!(seat.as_deref(), Some("Position 4 - At-Large"));
        assert_eq!(
            extract_office_district("Fort Bend School Board", seat.as_deref(), None, office),
            None
        );

        let (seat, _) = extract_office_seat("Fort Bend School Board Position 7", None, None, office);
        assert_eq!(seat.as_deref(), Some("Position 7"));
        assert_eq!(
            extract_office_district("Fort Bend School Board", seat.as_deref(), None, office),
            Some("East".to_string())
        );
    }
}
