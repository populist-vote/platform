use std::sync::OnceLock;

use regex::Regex;

use super::{default_capture, owned_capture};

static NAME_MATCHERS: OnceLock<Vec<(Regex, String)>> = OnceLock::new();
static TITLE_MATCHERS: OnceLock<Vec<(Regex, String)>> = OnceLock::new();
static CHAMBER_MATCHERS: OnceLock<Vec<(Regex, db::Chamber)>> = OnceLock::new();
static DISTRICT_TYPE_MATCHERS: OnceLock<Vec<(Regex, db::DistrictType)>> = OnceLock::new();
static POLITICAL_SCOPE_MATCHERS: OnceLock<Vec<(Regex, db::PoliticalScope)>> = OnceLock::new();
static ELECTION_SCOPE_MATCHERS: OnceLock<Vec<(Regex, db::ElectionScope)>> = OnceLock::new();

pub fn extract_office_name(input: &str) -> Option<String> {
    let matchers = NAME_MATCHERS.get_or_init(|| {
        [
            // Federal Offices
            (r"(?i:U(?:nited |.)S(?:tates|.) Senat(?:e|or))", "U.S. Senate".into()),
            (r"(?i:U(?:nited |.)S(?:tates|.) (?:House|Representative))", "U.S. House".into()),
            // State Offices
            (r"(?i:State Senat(?:e|or))", "State Senate".into()),
            (r"(?i:State (?:House|Representative))", "State House".into()),
            // County Offices
            (r"(?i:Soil and Water Supervisor)", "Soil and Water Supervisor".into()),
            (r"(?i:County Park Commissioner)", "County Park Commissioner".into()),
            (r"(?i:County Commissioner)", "County Commissioner".into()),
            // Judicial Offices
            (r"(?i:Chief Justice - Supreme Court)", "Chief Justice - Supreme Court".into()),
            (r"(?i:Associate Justice - Supreme Court)", "Associate Justice - Supreme Court".into()),
            (r"(?i:Judge - Court of Appeals)", "Judge - Court of Appeals".into()),
            (r"(?i:Judge - [0-9]{1,3}(?:st|nd|rd|th)? District)", "District Court Judge".into()),
            // Local Offices
            (r"(?i:Sanitary District Board Member)", "Sanitary District Board".into()),
            (r"(?i:Council Member)", "City Council".into()),
            (r"(?i:City Clerk - Treasurer)", "City Clerk & Treasurer".into()),
            (r"(?i:City Clerk)", "City Clerk".into()),
            (r"(?i:City Treasurer)", "City Treasurer".into()),
            (r"(?i:Mayor)", "Mayor".into()),
            (r"(?i:Town Clerk - Treasurer)", "Town Clerk & Treasurer".into()),
            (r"(?i:Town Clerk)", "Town Clerk".into()),
            (r"(?i:Town Treasurer)", "Town Treasurer".into()),
            (r"(?i:Town Supervisor)", "Town Supervisor".into()),
            (r"(?i:School Board Member)", "School Board".into()),
            (r"(?i:Hospital District Board Member)", "Hospital District Board".into()),
            (r"(?i:Utility Board Commissioner)", "Utility Board Commissioner".into()),
            (r"(?i:Board of Public Works)", "Board of Public Works".into()),
        ]
        .into_iter()
        .map(|t| (Regex::new(t.0).unwrap(), t.1))
        .collect()
    });

    for (matcher, name) in matchers {
        if matcher.is_match(input) {
            return Some(name.clone());
        }
    }
    None
}

pub fn extract_office_title(input: &str) -> Option<String> {
    let matchers = TITLE_MATCHERS.get_or_init(|| {
        [
            // Federal Offices
            (r"(?i:U(?:nited |.)S(?:tates|.) Senat(?:e|or))", "U.S. Senator".into()),
            (r"(?i:U(?:nited |.)S(?:tates|.) (?:House|Representative))", "U.S. Representative".into()),
            // State Offices
            (r"(?i:State Senat(?:e|or))", "State Senator".into()),
            (r"(?i:State (?:House|Representative))", "State Representative".into()),
            // County Offices
            (r"(?i:Soil and Water Supervisor)", "Soil and Water Supervisor".into()),
            (r"(?i:County Park Commissioner)", "County Park Commissioner".into()),
            (r"(?i:County Commissioner)", "County Commissioner".into()),
            // Judicial Offices
            (r"(?i:Chief Justice - Supreme Court)", "Chief Justice - Supreme Court".into()),
            (r"(?i:Associate Justice - Supreme Court)", "Associate Justice - Supreme Court".into()),
            (r"(?i:Judge - Court of Appeals)", "Judge - Court of Appeals".into()),
            (r"(?i:Judge - [0-9]{1,3}(?:st|nd|rd|th)? District)", "District Court Judge".into()),
            // Local Offices
            (r"(?i:Sanitary District Board Member)", "Sanitary District Board Member".into()),
            (r"(?i:Council Member)", "City Council Member".into()),
            (r"(?i:City Clerk - Treasurer)", "City Clerk & Treasurer".into()),
            (r"(?i:City Clerk)", "City Clerk".into()),
            (r"(?i:City Treasurer)", "City Treasurer".into()),
            (r"(?i:Mayor)", "Mayor".into()),
            (r"(?i:Town Clerk - Treasurer)", "Town Clerk & Treasurer".into()),
            (r"(?i:Town Clerk)", "Town Clerk".into()),
            (r"(?i:Town Treasurer)", "Town Treasurer".into()),
            (r"(?i:Town Supervisor)", "Town Supervisor".into()),
            (r"(?i:School Board Member)", "School Board Member".into()),
            (r"(?i:Hospital District Board Member)", "Hospital District Board Member".into()),
            (r"(?i:Utility Board Commissioner)", "Utility Board Commissioner".into()),
            (r"(?i:Board of Public Works)", "Board of Public Works Member".into()),
        ]
        .into_iter()
        .map(|t| (Regex::new(t.0).unwrap(), t.1))
        .collect()
    });

    for (matcher, title) in matchers {
        if matcher.is_match(input) {
            return Some(title.clone());
        }
    }
    None
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
            return Some(chamber);
        }
    }
    None
}

/// Extracts the district type from an office title.
/// Returns None (SQL NULL) if no matching district type is found.
/// For Soil and Water Supervisor, also checks if the county_id is in the allowed list, and returns None if not.
pub fn extract_office_district_type(input: &str, county_id: Option<i32>) -> Option<db::DistrictType> {
    // Special case for Soil and Water Supervisor - check county_id
    if input.to_lowercase().contains("soil and water supervisor") {
        let allowed_counties = [2, 10, 19, 56, 60, 62, 65, 69, 70, 82];
        if let Some(id) = county_id {
            if allowed_counties.contains(&id) {
                return Some(db::DistrictType::SoilAndWater);
            }
        }
        return None;
    }

    let matchers = DISTRICT_TYPE_MATCHERS.get_or_init(|| {
        [
            (r"(?i:U(?:nited |.)S(?:tates|.) (?:House|Representative))", db::DistrictType::UsCongressional),
            (r"(?i:State Senat(?:e|or))", db::DistrictType::StateSenate),
            (r"(?i:State (?:House|Representative))", db::DistrictType::StateHouse),
            (r"(?i:County (?:Commissioner|Park Commissioner))", db::DistrictType::County),
            (r"(?i:Council Member (?:Ward|District|Precinct|Section))", db::DistrictType::City),
            (r"(?i:School Board)", db::DistrictType::School),
            (r"(?i:District Court)", db::DistrictType::Judicial),
            (r"(?i:Hospital District Board)", db::DistrictType::Hospital),
        ]
        .into_iter()
        .map(|t| (Regex::new(t.0).unwrap(), t.1))
        .collect()
    });

    for (matcher, district_type) in matchers {
        if matcher.is_match(input) {
            return Some(district_type);
        }
    }
    // Return None (SQL NULL) if no match is found
    None
}

pub fn extract_office_political_scope(input: &str) -> Option<db::PoliticalScope> {
    let matchers = POLITICAL_SCOPE_MATCHERS.get_or_init(|| {
        [
            (r"(?i:U(?:nited |.)S(?:tates|.) (?:Senat(?:e|or)|House|Representative))", db::PoliticalScope::Federal),
            (r"(?i:State (?:Senat(?:e|or)|House|Representative))", db::PoliticalScope::State),
            (r"(?i:(?:Chief|Associate) Justice - Supreme Court|Judge - Court of Appeals)", db::PoliticalScope::State),
            (r"(?i:Judge - [0-9]{1,3}(?:st|nd|rd|th)? District)", db::PoliticalScope::Local),
            (r"(?i:(?:County|City|Town|School|Hospital|Utility|Sanitary|Public Works))", db::PoliticalScope::Local),
        ]
        .into_iter()
        .map(|t| (Regex::new(t.0).unwrap(), t.1))
        .collect()
    });

    for (matcher, scope) in matchers {
        if matcher.is_match(input) {
            return Some(scope);
        }
    }
    None
}

pub fn extract_office_election_scope(input: &str, county_id: Option<i32>) -> Option<db::ElectionScope> {
    let input_lower = input.to_lowercase();
    
    // County Offices
    if input_lower.contains("county attorney") ||
       input_lower.contains("county sheriff") ||
       input_lower.contains("county recorder") ||
       input_lower.contains("county surveyor") ||
       input_lower.contains("county coroner") ||
       input_lower.contains("county auditor/treasurer") ||
       input_lower.contains("county auditor") ||
       input_lower.contains("county treasurer") ||
       input_lower.contains("soil and water supervisor") {
        return Some(db::ElectionScope::County);
    }

    // District Offices
    if input_lower.contains("u.s. representative") ||
       input_lower.contains("state representative") ||
       input_lower.contains("state senator") ||
       input_lower.contains("county commissioner") ||
       input_lower.contains("county park commissioner") ||
       (input_lower.contains("judge") && input_lower.contains("district court")) ||
       (input_lower.contains("council member") && 
        (input_lower.contains("ward") || 
         input_lower.contains("district") || 
         input_lower.contains("precinct") || 
         input_lower.contains("section"))) ||
       input_lower.contains("school board member") {
        return Some(db::ElectionScope::District);
    }

    // Special case for Soil and Water Supervisor with county_id
    if input_lower.contains("soil and water supervisor") {
        let allowed_counties = [2, 10, 19, 56, 60, 62, 65, 69, 70, 82];
        if let Some(id) = county_id {
            if allowed_counties.contains(&id) {
                return Some(db::ElectionScope::District);
            }
        }
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

static DISTRICT_EXTRACTORS: OnceLock<Vec<Regex>> = OnceLock::new();

pub fn extract_office_district(input: &str) -> Option<String> {
    let extractors = DISTRICT_EXTRACTORS.get_or_init(|| {
        [
            r"District (\d+[A-Z]?|[A-Z]+)(?:\W|$)",
            r"(\d+)(?:st|nd|rd|th) (?:\w+ )?District",
        ]
        .into_iter()
        .map(|r| Regex::new(r).unwrap())
        .collect()
    });

    for extractor in extractors {
        if let Some(district) = extractor.captures(input).and_then(default_capture) {
            return Some(district);
        }
    }
    None
}

static SEAT_EXTRACTORS: OnceLock<Vec<Regex>> = OnceLock::new();

pub fn extract_office_seat(input: &str) -> Option<String> {
    let extractors = SEAT_EXTRACTORS.get_or_init(|| {
        [r"(?:\W|^)(?<atlarge>(?i)At Large)(?:\W|$)"]
            .into_iter()
            .map(|r| Regex::new(r).unwrap())
            .collect()
    });

    for extractor in extractors {
        if let Some(captures) = extractor.captures(input) {
            if captures.name("atlarge").is_some() {
                return Some("At Large".into());
            }
            if let Some(seat) = captures.get(1).map(owned_capture) {
                return Some(seat);
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
    let regex = Regex::new(r"Hospital District Board Member ([0-9]{1,2})").unwrap();
    if let Some(captures) = regex.captures(input) {
        return captures.get(1).map(owned_capture);
    }
    
    if input.contains("Hospital District Board Member at Large Koochiching") {
        return Some("Northern Itasca - Koochiching".into());
    }
    
    if input.contains("Hospital District Board Member at Large Itasca") {
        return Some("Northern Itasca - Itasca".into());
    }
    
    let regex = Regex::new(r"\(([^)]+)\)").unwrap();
    regex.captures(input)
        .and_then(|c| c.get(1))
        .map(owned_capture)
}

pub fn extract_municipality(input: &str, election_scope: &ElectionScope, district_type: &Option<DistrictType>) -> Option<String> {
    if election_scope == &ElectionScope::City {
        let regex = Regex::new(r"\(([^)]+)\)").unwrap();
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

        for (input, (name, title, chamber, district_type, political_scope, election_scope)) in tests {
            assert_eq!(extract_office_name(input), name, "Failed to extract name from: {}", input);
            assert_eq!(extract_office_title(input), title, "Failed to extract title from: {}", input);
            assert_eq!(extract_office_chamber(input), chamber, "Failed to extract chamber from: {}", input);
            assert_eq!(extract_office_district_type(input, None), district_type, "Failed to extract district type from: {}", input);
            assert_eq!(extract_office_political_scope(input), political_scope, "Failed to extract political scope from: {}", input);
            assert_eq!(extract_office_election_scope(input, None), election_scope, "Failed to extract election scope from: {}", input);
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
