use std::sync::OnceLock;

use regex::Regex;

use crate::extractors::{default_capture, owned_capture};
use db::{ElectionScope, DistrictType};

static CHAMBER_MATCHERS: OnceLock<Vec<(Regex, db::Chamber)>> = OnceLock::new();
static SEAT_EXTRACTORS: OnceLock<Vec<Regex>> = OnceLock::new();
static DISTRICT_EXTRACTORS: OnceLock<Vec<Regex>> = OnceLock::new();
static JUDICIAL_DISTRICT_REGEX: OnceLock<Regex> = OnceLock::new();
static HOSPITAL_DISTRICT_NUMBERED_REGEX: OnceLock<Regex> = OnceLock::new();
static HOSPITAL_DISTRICT_PAREN_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn extract_office_name(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    
    // Federal Offices
    if input_lower.contains("u.s. senator") || input_lower.contains("united states senator") {
        return Some("U.S. Senate".to_string());
    }
    if input_lower.contains("u.s. representative") || input_lower.contains("u.s. house") || 
       input_lower.contains("united states representative") {
        return Some("U.S. House".to_string());
    }
    
    // State Offices
    if input_lower.contains("state senator") || input_lower.contains("state senate") {
        return Some("State Senate".to_string());
    }
    if input_lower.contains("state representative") || input_lower.contains("state house") {
        return Some("State House".to_string());
    }
    
    // County Offices
    if input_lower.contains("soil and water supervisor") {
        return Some("Soil and Water Supervisor".to_string());
    }
    if input_lower.contains("county park commissioner") {
        return Some("County Park Commissioner".to_string());
    }
    if input_lower.contains("county commissioner") {
        return Some("County Commissioner".to_string());
    }
    
    // Judicial Offices
    if input_lower.contains("chief justice - supreme court") {
        return Some("Chief Justice - Supreme Court".to_string());
    }
    if input_lower.contains("associate justice - supreme court") {
        return Some("Associate Justice - Supreme Court".to_string());
    }
    if input_lower.contains("judge - court of appeals") {
        return Some("Judge - Court of Appeals".to_string());
    }
    
    // District Court Judge - extract full title with district number
    let judicial_regex = JUDICIAL_DISTRICT_REGEX.get_or_init(|| {
        Regex::new(r"(Judge - [0-9]{1,3}(st|nd|rd|th)? District)").unwrap()
    });
    if let Some(captures) = judicial_regex.captures(input) {
        if let Some(full_title) = captures.get(1) {
            return Some(full_title.as_str().to_string());
        }
    }
    
    // Local Offices
    if input_lower.contains("sanitary district board member") {
        return Some("Sanitary District Board".to_string());
    }
    if input_lower.contains("council member") {
        return Some("City Council".to_string());
    }
    if input_lower.contains("city clerk - treasurer") {
        return Some("City Clerk & Treasurer".to_string());
    }
    if input_lower.contains("city clerk") {
        return Some("City Clerk".to_string());
    }
    if input_lower.contains("city treasurer") {
        return Some("City Treasurer".to_string());
    }
    if input_lower.contains("mayor") {
        return Some("Mayor".to_string());
    }
    if input_lower.contains("town clerk - treasurer") {
        return Some("Town Clerk & Treasurer".to_string());
    }
    if input_lower.contains("town clerk") {
        return Some("Town Clerk".to_string());
    }
    if input_lower.contains("town treasurer") {
        return Some("Town Treasurer".to_string());
    }
    if input_lower.contains("town supervisor") {
        return Some("Town Supervisor".to_string());
    }
    if input_lower.contains("school board member") {
        return Some("School Board".to_string());
    }
    if input_lower.contains("hospital district board member") {
        return Some("Hospital District Board".to_string());
    }
    if input_lower.contains("utility board commissioner") {
        return Some("Utility Board Commissioner".to_string());
    }
    if input_lower.contains("board of public works") {
        return Some("Board of Public Works".to_string());
    }
    if input_lower.contains("board of estimate and taxation") {
        return Some("Board of Estimate and Taxation".to_string());
    }
    if input_lower.contains("park and recreation commissioner") {
        return Some("Park and Recreation Commissioner".to_string());
    }
    
    // Fallback - return input as-is
    Some(input.to_string())
}

pub fn extract_office_title(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    
    // Federal Offices
    if input_lower.contains("u.s. senator") || input_lower.contains("united states senator") {
        return Some("U.S. Senator".to_string());
    }
    if input_lower.contains("u.s. representative") || input_lower.contains("u.s. house") || 
       input_lower.contains("united states representative") {
        return Some("U.S. Representative".to_string());
    }
    
    // State Offices
    if input_lower.contains("state senator") || input_lower.contains("state senate") {
        return Some("State Senator".to_string());
    }
    if input_lower.contains("state representative") || input_lower.contains("state house") {
        return Some("State Representative".to_string());
    }
    
    // County Offices
    if input_lower.contains("soil and water supervisor") {
        return Some("Soil and Water Supervisor".to_string());
    }
    if input_lower.contains("county park commissioner") {
        return Some("County Park Commissioner".to_string());
    }
    if input_lower.contains("county commissioner") {
        return Some("County Commissioner".to_string());
    }
    
    // Judicial Offices
    if input_lower.contains("chief justice - supreme court") {
        return Some("Chief Justice - Supreme Court".to_string());
    }
    if input_lower.contains("associate justice - supreme court") {
        return Some("Associate Justice - Supreme Court".to_string());
    }
    if input_lower.contains("judge - court of appeals") {
        return Some("Judge - Court of Appeals".to_string());
    }
    
    // District Court Judge - extract full title with district number (same as name)
    let judicial_regex = JUDICIAL_DISTRICT_REGEX.get_or_init(|| {
        Regex::new(r"(Judge - [0-9]{1,3}(st|nd|rd|th)? District)").unwrap()
    });
    if let Some(captures) = judicial_regex.captures(input) {
        if let Some(full_title) = captures.get(1) {
            return Some(full_title.as_str().to_string());
        }
    }
    
    // Local Offices
    if input_lower.contains("council member") {
        return Some("City Council Member".to_string());
    }
    if input_lower.contains("city clerk - treasurer") {
        return Some("City Clerk & Treasurer".to_string());
    }
    if input_lower.contains("city clerk") {
        return Some("City Clerk".to_string());
    }
    if input_lower.contains("city treasurer") {
        return Some("City Treasurer".to_string());
    }
    if input_lower.contains("mayor") {
        return Some("Mayor".to_string());
    }
    if input_lower.contains("town clerk - treasurer") {
        return Some("Town Clerk & Treasurer".to_string());
    }
    if input_lower.contains("town clerk") {
        return Some("Town Clerk".to_string());
    }
    if input_lower.contains("town treasurer") {
        return Some("Town Treasurer".to_string());
    }
    if input_lower.contains("town supervisor") {
        return Some("Town Supervisor".to_string());
    }
    if input_lower.contains("school board member") {
        return Some("School Board Member".to_string());
    }
    if input_lower.contains("hospital district board member") {
        return Some("Hospital District Board Member".to_string());
    }
    if input_lower.contains("utility board commissioner") {
        return Some("Utility Board Commissioner".to_string());
    }
    if input_lower.contains("board of public works") {
        return Some("Board of Public Works Member".to_string());
    }
    if input_lower.contains("sanitary district board member") {
        return Some("Sanitary District Board Member".to_string());
    }
    if input_lower.contains("board of estimate and taxation") {
        return Some("Board of Estimate and Taxation Member".to_string());
    }
    if input_lower.contains("park and recreation commissioner") {
        return Some("Park and Recreation Commissioner".to_string());
    }
    
    // Fallback - return input as-is
    Some(input.to_string())
}

pub fn extract_office_chamber(input: &str) -> Option<db::Chamber> {
    let matchers = CHAMBER_MATCHERS.get_or_init(|| {
        [
            (r"(?i:U(?:nited |.)S(?:tates|.) Senat(?:e|or))", db::Chamber::Senate),
            (r"(?i:U(?:nited |.)S(?:tates|.) (?:House|Representative))", db::Chamber::House),
            (r"(?i:State Senat(?:e|or))", db::Chamber::Senate),
            (r"(?i:State (?:House|Representative))", db::Chamber::House),
        ]
        .into_iter()
        .map(|t| (Regex::new(t.0).unwrap(), t.1))
        .collect()
    });

    for (matcher, chamber) in matchers {
        if matcher.is_match(input) {
            return Some(*chamber);
        }
    }
    None
}

/// Extracts the district type from an office title.
/// Returns None (SQL NULL) if no matching district type is found.
/// For Soil and Water Supervisor, also checks if the county_id is in the allowed list, and returns None if not.
pub fn extract_office_district_type(input: &str, county_id: Option<i32>) -> Option<db::DistrictType> {
    let input_lower = input.to_lowercase();
    
    // U.S. Representative
    if input_lower.contains("u.s. representative") {
        return Some(db::DistrictType::UsCongressional);
    }
    
    // State Senator
    if input_lower.contains("state senator") {
        return Some(db::DistrictType::StateSenate);
    }
    
    // State Representative
    if input_lower.contains("state representative") {
        return Some(db::DistrictType::StateHouse);
    }
    
    // Soil and Water Supervisor - only if county_id is in allowed list
    if input_lower.contains("soil and water supervisor") {
        let allowed_counties = [2, 10, 19, 56, 60, 62, 65, 69, 70, 82];
        if let Some(id) = county_id {
            if allowed_counties.contains(&id) {
                return Some(db::DistrictType::SoilAndWater);
            }
        }
        // If not in allowed counties or no county_id, return None
        return None;
    }
    
    // County Commissioner
    if input_lower.contains("county commissioner") {
        return Some(db::DistrictType::County);
    }
    
    // County Park Commissioner
    if input_lower.contains("county park commissioner") {
        return Some(db::DistrictType::County);
    }
    
    // Council Member Ward/District/Precinct/Section
    if input_lower.contains("council member ward") ||
       input_lower.contains("council member district") ||
       input_lower.contains("council member precinct") ||
       input_lower.contains("council member section") {
        return Some(db::DistrictType::City);
    }
    
    // School Board
    if input_lower.contains("school board") {
        return Some(db::DistrictType::School);
    }
    
    // District Court
    if input_lower.contains("district court") {
        return Some(db::DistrictType::Judicial);
    }
    
    // Hospital District Board
    if input_lower.contains("hospital district board") {
        return Some(db::DistrictType::Hospital);
    }

    // Park and Recreation Commissioner
    if input_lower.contains("park and recreation") {
        return Some(db::DistrictType::Park);
    }
    
    // Return None (SQL NULL) if no match is found
    None
}

/// Extracts the political scope based on election scope, office name, and district type
/// 
/// This matches the logic from dbt/macros/get_political_scope.sql
/// 
/// # Arguments
/// * `office_name` - The normalized office name (e.g., "U.S. Senate", "Mayor")
/// * `election_scope` - The election scope (National, State, District, County, City)
/// * `district_type` - The district type (UsCongressional, StateSenate, etc.)
/// 
/// # Returns
/// * The political scope (Federal, State, or Local)
pub fn extract_office_political_scope(
    office_name: Option<&str>,
    election_scope: &db::ElectionScope,
    district_type: &Option<db::DistrictType>,
) -> db::PoliticalScope {
    use db::{ElectionScope, DistrictType, PoliticalScope};
    
    match election_scope {
        // National scope is always Federal
        ElectionScope::National => PoliticalScope::Federal,
        
        // State scope is always State
        ElectionScope::State => {
            // Special check for U.S. Senate which might be marked as State scope
            if let Some(name) = office_name {
                if name.to_lowercase().contains("u.s. senate") {
                    return PoliticalScope::Federal;
                }
            }
            PoliticalScope::State
        },
        
        // District scope depends on district type
        ElectionScope::District => {
            match district_type {
                Some(DistrictType::UsCongressional) => {
                    // Check if it's actually a U.S. office or a state office in a congressional district
                    if let Some(name) = office_name {
                        if name.starts_with("U.S.") {
                            PoliticalScope::Federal
                        } else {
                            PoliticalScope::State
                        }
                    } else {
                        PoliticalScope::Federal
                    }
                },
                Some(DistrictType::StateSenate) | Some(DistrictType::StateHouse) => PoliticalScope::State,
                _ => PoliticalScope::Local,
            }
        },
        
        // County and City scopes are always Local
        ElectionScope::County | ElectionScope::City => PoliticalScope::Local,
    }
}

pub fn extract_office_election_scope(input: &str, county_id: Option<i32>) -> Option<db::ElectionScope> {
    let input_lower = input.to_lowercase();
    
    // Special case for Soil and Water Supervisor - must be checked BEFORE general county offices
    if input_lower.contains("soil and water supervisor") {
        let allowed_counties = [2, 10, 19, 56, 60, 62, 65, 69, 70, 82];
        if let Some(id) = county_id {
            if allowed_counties.contains(&id) {
                return Some(db::ElectionScope::District);
            }
        }
        // For counties not in allowed list, it's County scope
        return Some(db::ElectionScope::County);
    }

    // County Offices
    if input_lower.contains("county attorney") ||
       input_lower.contains("county sheriff") ||
       input_lower.contains("county recorder") ||
       input_lower.contains("county surveyor") ||
       input_lower.contains("county coroner") ||
       input_lower.contains("county auditor/treasurer") ||
       input_lower.contains("county auditor") ||
       input_lower.contains("county treasurer") {
        return Some(db::ElectionScope::County);
    }

    // District Offices
    if input_lower.contains("u.s. representative") ||
       input_lower.contains("state representative") ||
       input_lower.contains("state senator") ||
       input_lower.contains("county commissioner") ||
       input_lower.contains("county park commissioner") ||
       (input_lower.contains("park and recreation commissioner") && input_lower.contains("district")) ||
       (input_lower.contains("judge") && input_lower.contains("district court")) ||
       (input_lower.contains("council member") && 
        (input_lower.contains("ward") || 
         input_lower.contains("district") || 
         input_lower.contains("precinct") || 
         input_lower.contains("section"))) ||
       input_lower.contains("school board member") {
        return Some(db::ElectionScope::District);
    }

    // City Offices
    if input_lower.contains("city clerk - treasurer") ||
       input_lower.contains("city clerk") ||
       input_lower.contains("city treasurer") ||
       (input_lower.contains("council member") && 
        !input_lower.contains("ward") && 
        !input_lower.contains("district") && 
        !input_lower.contains("precinct") && 
        !input_lower.contains("section")) ||
       input_lower.contains("mayor") ||
       input_lower.contains("town clerk - treasurer") ||
       input_lower.contains("town clerk") ||
       input_lower.contains("town treasurer") ||
       input_lower.contains("town supervisor") ||
       input_lower.contains("sanitary district board") ||
       input_lower.contains("board of public works") ||
       input_lower.contains("utility board commissioner") ||
       input_lower.contains("board of estimate and taxation") ||
       (input_lower.contains("park and recreation commissioner") && input_lower.contains("at large")) ||
       input_lower.contains("police chief") {
        return Some(db::ElectionScope::City);
    }

    // Special case for Hospital District Board
    if input_lower.contains("hospital district board") {
        if input_lower.contains("at large") || input_lower.contains("(cook county)") {
            return Some(db::ElectionScope::District);
        }
        return Some(db::ElectionScope::City);
    }

    // State Offices
    if input_lower.contains("u.s. senator") ||
       input_lower.contains("governor") ||
       input_lower.contains("lieutenant governor") ||
       input_lower.contains("secretary of state") ||
       input_lower.contains("attorney general") ||
       input_lower.contains("treasurer") ||
       input_lower.contains("state auditor") ||
       input_lower.contains("board of education") ||
       input_lower.contains("supreme court") ||
       input_lower.contains("court of appeals") {
        return Some(db::ElectionScope::State);
    }

    // Default to state as per SQL
    Some(db::ElectionScope::State)
}

pub fn extract_office_district(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    
    // Initialize regex patterns once
    let extractors = DISTRICT_EXTRACTORS.get_or_init(|| {
        vec![
            Regex::new(r"District ([0-9]{1,3}[A-Z]?)").unwrap(),           // District 1, District 62A
            Regex::new(r"([0-9]{1,3})(st|nd|rd|th) District").unwrap(),    // 1st District, 2nd District
            Regex::new(r"Ward ([0-9A-Z]+)").unwrap(),                      // Ward 3, Ward 5A
            Regex::new(r"Wards ([0-9]{1,3} & [0-9]{1,3})").unwrap(),       // Wards 1 & 2
            Regex::new(r"Precinct ([0-9]{1,3})").unwrap(),                 // Precinct 3
            Regex::new(r"Section ([I|II]+)").unwrap(),                     // Section I, Section II
            Regex::new(r"Board Member ([0-9]{1,3})").unwrap(),             // Hospital Board Member 1
            Regex::new(r"Position ([0-9]{1,3})").unwrap(),                 // School Board Position 2
        ]
    });
    
    // Soil & Water Supervisor Districts with cardinal directions (North, South, East, West)
    if input_lower.contains("soil and water supervisor") {
        // Check for cardinal directions in parentheses
        if input_lower.contains("(north)") {
            if let Some(captures) = extractors[0].captures(input) {
                if let Some(district) = captures.get(1) {
                    return Some(format!("{} (North)", district.as_str()));
                }
            }
        } else if input_lower.contains("(south)") {
            if let Some(captures) = extractors[0].captures(input) {
                if let Some(district) = captures.get(1) {
                    return Some(format!("{} (South)", district.as_str()));
                }
            }
        } else if input_lower.contains("(east)") {
            if let Some(captures) = extractors[0].captures(input) {
                if let Some(district) = captures.get(1) {
                    return Some(format!("{} (East)", district.as_str()));
                }
            }
        } else if input_lower.contains("(west)") {
            if let Some(captures) = extractors[0].captures(input) {
                if let Some(district) = captures.get(1) {
                    return Some(format!("{} (West)", district.as_str()));
                }
            }
        }
    }
    
    // City Council Wards - Red Wing special case
    if input_lower.contains("council member wards") && input_lower.contains("red wing") {
        if let Some(captures) = extractors[3].captures(input) {
            if let Some(wards) = captures.get(1) {
                return Some(format!("Wards {}", wards.as_str()));
            }
        }
    }
    
    // City Council Ward (standard case)
    if input_lower.contains("council member ward") {
        if let Some(captures) = extractors[2].captures(input) {
            if let Some(ward) = captures.get(1) {
                return Some(format!("Ward {}", ward.as_str()));
            }
        }
    }
    
    // City Council Precinct - Glencoe special case
    if input_lower.contains("council member precinct") && input_lower.contains("glencoe") {
        if let Some(captures) = extractors[4].captures(input) {
            if let Some(precinct) = captures.get(1) {
                return Some(format!("Precinct {}", precinct.as_str()));
            }
        }
    }
    
    // City Council Section - Crystal special case
    if input_lower.contains("council member section") && input_lower.contains("crystal") {
        if let Some(captures) = extractors[5].captures(input) {
            if let Some(section) = captures.get(1) {
                return Some(format!("Section {}", section.as_str()));
            }
        }
    }
    
    // School Board Member - specific district names
    if input_lower.contains("school board member fairfax district") {
        return Some("Fairfax District".to_string());
    }
    if input_lower.contains("school board member gibbon district") {
        return Some("Gibbon District".to_string());
    }
    if input_lower.contains("school board member winthrop district") {
        return Some("Winthrop District".to_string());
    }
    if input_lower.contains("school board member russell district") {
        return Some("Russell District".to_string());
    }
    if input_lower.contains("school board member tyler district") {
        return Some("Tyler District".to_string());
    }
    if input_lower.contains("school board member ruthton district") {
        return Some("Ruthton District".to_string());
    }
    
    // ISD #861 and #390 use Position # as District #
    if input_lower.contains("school board member position") {
        if input_lower.contains("isd #861") || input_lower.contains("isd #390") {
            if let Some(captures) = extractors[7].captures(input) {
                if let Some(position) = captures.get(1) {
                    return Some(position.as_str().to_string());
                }
            }
        }
    }
    
    // Hospital Districts - Cook County subdistricts
    if input_lower.contains("hospital district board") && input_lower.contains("(cook county)") {
        if let Some(captures) = extractors[6].captures(input) {
            if let Some(member_num) = captures.get(1) {
                return Some(member_num.as_str().to_string());
            }
        }
    }
    
    // Judicial districts (Xth District format)
    if let Some(captures) = extractors[1].captures(input) {
        if let Some(district) = captures.get(1) {
            return Some(district.as_str().to_string());
        }
    }
    
    // Generic District pattern (most common - try last to avoid false matches)
    if let Some(captures) = extractors[0].captures(input) {
        if let Some(district) = captures.get(1) {
            return Some(district.as_str().to_string());
        }
    }
    
    None
}

pub fn extract_office_seat(input: &str) -> Option<String> {
    let input_lower = input.to_lowercase();
    
    // Special case: U.S. Senator always gets seat "1"
    // if input.eq_ignore_ascii_case("U.S. Senator") {
    //     return Some("1".to_string());
    // }
    
    // At Large
    if input_lower.contains("at large") {
        return Some("At Large".to_string());
    }

    // Initialize regex extractors once
    let extractors = SEAT_EXTRACTORS.get_or_init(|| {
        vec![
            Regex::new(r"Seat ([A-Za-z0-9]+)").unwrap(),           // Seat A, Seat 1, Seat 2B
            Regex::new(r"Court ([0-9]{1,3})").unwrap(),            // District Court 4, Supreme Court 2
            Regex::new(r"Appeals ([0-9]{1,3})").unwrap(),          // Court of Appeals 5
            Regex::new(r"Position ([0-9]{1,3})").unwrap(),         // School Board Position 2
        ]
    });
    
    // Council Member with Seat edge case
    if input_lower.contains("council member") && input_lower.contains("seat") {
        if let Some(seat) = extractors[0].captures(input).and_then(|c| c.get(1)) {
            return Some(format!("At Large - {}", seat.as_str()));
        }
        // If has a seat number, this should mean that the council member is an at large seat
    }
    
    // Seat [A-Za-z0-9]+
    if input_lower.contains("seat") {
        if let Some(seat) = extractors[0].captures(input).and_then(|c| c.get(1)) {
            return Some(seat.as_str().to_string());
        }
    }
    
    // District Court or Supreme Court - extract court number
    if input_lower.contains("district court") || input_lower.contains("supreme court") {
        if let Some(court_num) = extractors[1].captures(input).and_then(|c| c.get(1)) {
            return Some(court_num.as_str().to_string());
        }
    }
    
    // Court of Appeals - extract appeals court number
    if input_lower.contains("court of appeals") {
        if let Some(appeals_num) = extractors[2].captures(input).and_then(|c| c.get(1)) {
            return Some(appeals_num.as_str().to_string());
        }
    }
    
    // School Board Member Position (special cases for ISD #535 and ISD #206)
    if input_lower.contains("school board member position") {
        if input_lower.contains("isd #535") || input_lower.contains("isd #206") {
            if let Some(position) = extractors[3].captures(input).and_then(|c| c.get(1)) {
                return Some(position.as_str().to_string());
            }
        }
    }
    
    None
}

pub fn extract_school_district(input: &str) -> Option<String> {
    let regex = Regex::new(r"\((SSD #[0-9]+|ISD #[0-9]+)\)").unwrap();
    regex.captures(input)
        .and_then(|c| c.get(1))
        .map(owned_capture)
}

pub fn extract_hospital_district(input: &str) -> Option<String> {
    let paren_regex = HOSPITAL_DISTRICT_PAREN_REGEX.get_or_init(|| {
        Regex::new(r"\(([^)]+)\)").unwrap()
    });
    
    // Check for "Hospital District Board Member ([0-9]{1,2})" pattern
    // If matched, extract content from parentheses (not the number)
    let numbered_regex = HOSPITAL_DISTRICT_NUMBERED_REGEX.get_or_init(|| {
        Regex::new(r"Hospital District Board Member [0-9]{1,2}").unwrap()
    });
    if numbered_regex.is_match(input) {
        if let Some(captures) = paren_regex.captures(input) {
            return captures.get(1).map(owned_capture);
        }
    }
    
    // Specific "at Large" cases
    if input.contains("Hospital District Board Member at Large Koochiching") {
        return Some("Northern Itasca - Koochiching".into());
    }
    
    if input.contains("Hospital District Board Member at Large Itasca") {
        return Some("Northern Itasca - Itasca".into());
    }
    
    // Generic "at Large" case - extract from parentheses
    if input.contains("Hospital District Board Member at Large") {
        if let Some(captures) = paren_regex.captures(input) {
            return captures.get(1).map(owned_capture);
        }
    }
    
    // No match found - return None
    None
}

static MUNICIPALITY_REGEX: OnceLock<Regex> = OnceLock::new();

pub fn extract_municipality(input: &str, _election_scope: &ElectionScope, _district_type: &Option<DistrictType>) -> Option<String> {
    let input_lower = input.to_lowercase();
    
    // City Offices - extract municipality from parentheses
    let should_extract = 
        input_lower.contains("city clerk - treasurer") ||
        input_lower.contains("city clerk") ||
        input_lower.contains("city treasurer") ||
        input_lower.contains("council member") ||
        input_lower.contains("mayor") ||
        input_lower.contains("town clerk - treasurer") ||
        input_lower.contains("town clerk") ||
        input_lower.contains("town treasurer") ||
        input_lower.contains("town supervisor") ||
        input_lower.contains("sanitary district board") ||
        input_lower.contains("board of public works") ||
        input_lower.contains("utility board commissioner") ||
        input_lower.contains("police chief") ||
        input_lower.contains("board of estimate and taxation") ||
        // Hospital District Board - exclude At Large and Cook County
        (input_lower.contains("hospital district board") && 
         !input_lower.contains("at large") && 
         !input_lower.contains("(cook county)")) ||
        // Park and Recreation Commissioner
        input_lower.contains("park and recreation commissioner");
    
    if should_extract {
        let regex = MUNICIPALITY_REGEX.get_or_init(|| Regex::new(r"\(([^)]+)\)").unwrap());
        return regex.captures(input)
            .and_then(|c| c.get(1))
            .map(owned_capture);
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_office_attributes() {
        let tests = vec![
            (
                "U.S. Senator",
                (
                    Some("U.S. Senate"),
                    Some("U.S. Senator"),
                    Some(db::Chamber::Senate),
                    Some(db::DistrictType::StateSenate),
                    Some(db::PoliticalScope::Federal),
                    Some(db::ElectionScope::State),
                ),
            ),
            (
                "State Representative",
                (
                    Some("State House"),
                    Some("State Representative"),
                    Some(db::Chamber::House),
                    Some(db::DistrictType::StateHouse),
                    Some(db::PoliticalScope::State),
                    Some(db::ElectionScope::District),
                ),
            ),
            (
                "Mayor",
                (
                    Some("Mayor"),
                    Some("Mayor"),
                    None,
                    None,
                    Some(db::PoliticalScope::Local),
                    Some(db::ElectionScope::City),
                ),
            ),
        ];

        for (input, (expected_name, expected_title, expected_chamber, expected_district_type, expected_political_scope, expected_election_scope)) in tests {
            let name = extract_office_name(input);
            let title = extract_office_title(input);
            let chamber = extract_office_chamber(input);
            let district_type = extract_office_district_type(input, None);
            let election_scope = extract_office_election_scope(input, None);
            
            // Political scope now requires election_scope, name, and district_type
            let political_scope = if let Some(scope) = election_scope {
                Some(extract_office_political_scope(name.as_deref(), &scope, &district_type))
            } else {
                None
            };
            
            assert_eq!(name, expected_name, "Failed to extract name from: {}", input);
            assert_eq!(title, expected_title, "Failed to extract title from: {}", input);
            assert_eq!(chamber, expected_chamber, "Failed to extract chamber from: {}", input);
            assert_eq!(district_type, expected_district_type, "Failed to extract district type from: {}", input);
            assert_eq!(political_scope, expected_political_scope, "Failed to extract political scope from: {}", input);
            assert_eq!(election_scope, expected_election_scope, "Failed to extract election scope from: {}", input);
        }
    }

    #[test]
    fn extract_district() {
        let tests: Vec<(&'static str, Option<&'static str>)> = vec![
            ("District", None),
            ("District ", None),
            ("District Attorney", None),
            ("District 0", Some("0")),
            ("Something District 1", Some("1")),
            ("District 12", Some("12")),
            ("District A", Some("A")),
            ("District AB", Some("AB")),
            ("District 1A", Some("1A")),
            (" District 01 ", Some("01")), // TODO weird edge case?
            // ----
            ("1st", None),
            ("1st District", Some("1")),
            ("2nd District", Some("2")),
            ("3rd District", Some("3")),
            ("4th District", Some("4")),
            ("5th District", Some("5")),
            ("15th District", Some("15")),
            ("2nd Something District", Some("2")),
            (" 01th Something District ", Some("01")), // TODO weird edge case?
        ];

        for (input, expected) in tests {
            assert_eq!(
                extract_office_district(input).as_ref().map(String::as_str),
                expected,
                "\n\n  Test Case: '{input}'\n"
            );
        }
    }

    #[test]
    fn extract_seat() {
        let tests: Vec<(&'static str, Option<&'static str>)> = vec![
            ("fat largemouth bass", None),
            ("At Large.", Some("At Large")),
            (" at large", Some("At Large")),
        ];

        for (input, expected) in tests {
            assert_eq!(
                extract_office_seat(input).as_ref().map(String::as_str),
                expected,
                "\n\n  Test Case: '{input}'\n"
            );
        }
    }

    #[test]
    fn extract_district_type() {
        let tests = vec![
            // Federal Offices
            ("U.S. Representative", None, Some(db::DistrictType::UsCongressional)),
            ("United States Representative", None, Some(db::DistrictType::UsCongressional)),
            ("U.S. House", None, Some(db::DistrictType::UsCongressional)),
            
            // State Offices
            ("State Senator", None, Some(db::DistrictType::StateSenate)),
            ("State Senate", None, Some(db::DistrictType::StateSenate)),
            ("State Representative", None, Some(db::DistrictType::StateHouse)),
            ("State House", None, Some(db::DistrictType::StateHouse)),
            
            // County Offices - Soil and Water Supervisor with county_id
            ("Soil and Water Supervisor", Some(2), Some(db::DistrictType::SoilAndWater)),
            ("Soil and Water Supervisor", Some(10), Some(db::DistrictType::SoilAndWater)),
            ("Soil and Water Supervisor", Some(19), Some(db::DistrictType::SoilAndWater)),
            ("Soil and Water Supervisor", Some(56), Some(db::DistrictType::SoilAndWater)),
            ("Soil and Water Supervisor", Some(60), Some(db::DistrictType::SoilAndWater)),
            ("Soil and Water Supervisor", Some(62), Some(db::DistrictType::SoilAndWater)),
            ("Soil and Water Supervisor", Some(65), Some(db::DistrictType::SoilAndWater)),
            ("Soil and Water Supervisor", Some(69), Some(db::DistrictType::SoilAndWater)),
            ("Soil and Water Supervisor", Some(70), Some(db::DistrictType::SoilAndWater)),
            ("Soil and Water Supervisor", Some(82), Some(db::DistrictType::SoilAndWater)),
            // Soil and Water Supervisor with invalid county_id
            ("Soil and Water Supervisor", Some(1), None),
            ("Soil and Water Supervisor", Some(3), None),
            ("Soil and Water Supervisor", None, None),
            
            // Other County Offices
            ("County Commissioner", None, Some(db::DistrictType::County)),
            ("County Park Commissioner", None, Some(db::DistrictType::County)),
            
            // City Offices
            ("Council Member Ward 1", None, Some(db::DistrictType::City)),
            ("Council Member District 2", None, Some(db::DistrictType::City)),
            ("Council Member Precinct 3", None, Some(db::DistrictType::City)),
            ("Council Member Section 4", None, Some(db::DistrictType::City)),
            
            // School and Hospital
            ("School Board Member", None, Some(db::DistrictType::School)),
            ("Hospital District Board Member", None, Some(db::DistrictType::Hospital)),
            
            // Judicial
            ("District Court Judge", None, Some(db::DistrictType::Judicial)),
            
            // Non-matching cases
            ("U.S. Senator", None, None),
            ("Mayor", None, None),
            ("City Clerk", None, None),
            ("Town Supervisor", None, None),
        ];

        for (input, county_id, expected) in tests {
            assert_eq!(
                extract_office_district_type(input, county_id),
                expected,
                "\n\n  Test Case: '{input}' with county_id {:?}\n",
                county_id
            );
        }
    }

    #[test]
    fn extract_election_scope() {
        let tests = vec![
            // County Offices
            ("County Attorney", None, Some(db::ElectionScope::County)),
            ("County Sheriff", None, Some(db::ElectionScope::County)),
            ("County Recorder", None, Some(db::ElectionScope::County)),
            ("County Surveyor", None, Some(db::ElectionScope::County)),
            ("County Coroner", None, Some(db::ElectionScope::County)),
            ("County Auditor/Treasurer", None, Some(db::ElectionScope::County)),
            ("County Auditor", None, Some(db::ElectionScope::County)),
            ("County Treasurer", None, Some(db::ElectionScope::County)),
            ("Soil and Water Supervisor", None, Some(db::ElectionScope::County)),
            
            // District Offices
            ("U.S. Representative", None, Some(db::ElectionScope::District)),
            ("State Representative", None, Some(db::ElectionScope::District)),
            ("State Senator", None, Some(db::ElectionScope::District)),
            ("County Commissioner", None, Some(db::ElectionScope::District)),
            ("County Park Commissioner", None, Some(db::ElectionScope::District)),
            ("Judge - District Court", None, Some(db::ElectionScope::District)),
            ("Council Member Ward 1", None, Some(db::ElectionScope::District)),
            ("Council Member District 2", None, Some(db::ElectionScope::District)),
            ("Council Member Precinct 3", None, Some(db::ElectionScope::District)),
            ("Council Member Section 4", None, Some(db::ElectionScope::District)),
            ("School Board Member", None, Some(db::ElectionScope::District)),
            
            // Special case for Soil and Water Supervisor with county_id
            ("Soil and Water Supervisor", Some(2), Some(db::ElectionScope::District)),
            ("Soil and Water Supervisor", Some(10), Some(db::ElectionScope::District)),
            ("Soil and Water Supervisor", Some(19), Some(db::ElectionScope::District)),
            ("Soil and Water Supervisor", Some(56), Some(db::ElectionScope::District)),
            ("Soil and Water Supervisor", Some(60), Some(db::ElectionScope::District)),
            ("Soil and Water Supervisor", Some(62), Some(db::ElectionScope::District)),
            ("Soil and Water Supervisor", Some(65), Some(db::ElectionScope::District)),
            ("Soil and Water Supervisor", Some(69), Some(db::ElectionScope::District)),
            ("Soil and Water Supervisor", Some(70), Some(db::ElectionScope::District)),
            ("Soil and Water Supervisor", Some(82), Some(db::ElectionScope::District)),
            ("Soil and Water Supervisor", Some(1), Some(db::ElectionScope::County)),
            
            // City Offices
            ("City Clerk - Treasurer", None, Some(db::ElectionScope::City)),
            ("City Clerk", None, Some(db::ElectionScope::City)),
            ("City Treasurer", None, Some(db::ElectionScope::City)),
            ("Council Member", None, Some(db::ElectionScope::City)),
            ("Mayor", None, Some(db::ElectionScope::City)),
            ("Town Clerk - Treasurer", None, Some(db::ElectionScope::City)),
            ("Town Clerk", None, Some(db::ElectionScope::City)),
            ("Town Treasurer", None, Some(db::ElectionScope::City)),
            ("Town Supervisor", None, Some(db::ElectionScope::City)),
            ("Sanitary District Board", None, Some(db::ElectionScope::City)),
            ("Board of Public Works", None, Some(db::ElectionScope::City)),
            ("Utility Board Commissioner", None, Some(db::ElectionScope::City)),
            ("Police Chief", None, Some(db::ElectionScope::City)),
            
            // Hospital District Board special cases
            ("Hospital District Board Member at Large", None, Some(db::ElectionScope::District)),
            ("Hospital District Board Member (Cook County)", None, Some(db::ElectionScope::District)),
            ("Hospital District Board Member", None, Some(db::ElectionScope::City)),
            
            // State Offices
            ("U.S. Senator", None, Some(db::ElectionScope::State)),
            ("Governor", None, Some(db::ElectionScope::State)),
            ("Lieutenant Governor", None, Some(db::ElectionScope::State)),
            ("Secretary of State", None, Some(db::ElectionScope::State)),
            ("Attorney General", None, Some(db::ElectionScope::State)),
            ("Treasurer", None, Some(db::ElectionScope::State)),
            ("State Auditor", None, Some(db::ElectionScope::State)),
            ("Board of Education", None, Some(db::ElectionScope::State)),
            ("Supreme Court Justice", None, Some(db::ElectionScope::State)),
            ("Court of Appeals Judge", None, Some(db::ElectionScope::State)),
            
            // Default case
            ("Unknown Office", None, Some(db::ElectionScope::State)),
        ];

        for (input, county_id, expected) in tests {
            assert_eq!(
                extract_office_election_scope(input, county_id),
                expected,
                "\n\n  Test Case: '{input}' with county_id {:?}\n",
                county_id
            );
        }
    }
}
