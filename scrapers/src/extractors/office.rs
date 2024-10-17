use std::sync::OnceLock;

use regex::Regex;

use super::{default_capture, owned_capture};

#[derive(Clone, Debug, PartialEq)]
pub struct OfficeMeta {
    pub name: String,
    pub title: String,
    pub chamber: Option<db::Chamber>,
    pub district_type: Option<db::DistrictType>,
    pub political_scope: db::PoliticalScope,
    pub election_scope: db::ElectionScope,
}

static MATCHERS: OnceLock<Vec<(Regex, OfficeMeta)>> = OnceLock::new();

pub fn extract_office_meta(input: &str) -> Option<OfficeMeta> {
    let matchers = MATCHERS.get_or_init(|| {
        [
            (
                r"(?i:U(?:nited |.)S(?:tates|.) Senat(?:e|or))",
                OfficeMeta {
                    name: "U.S. Senate".into(),
                    title: "U.S. Senator".into(),
                    chamber: Some(db::Chamber::Senate),
                    district_type: Some(db::DistrictType::StateSenate),
                    political_scope: db::PoliticalScope::Federal,
                    election_scope: db::ElectionScope::State,
                },
            ),
            (
                r"(?i:U(?:nited |.)S(?:tates|.) (?:House|Representative))",
                OfficeMeta {
                    name: "U.S. House".into(),
                    title: "U.S. Representative".into(),
                    chamber: Some(db::Chamber::House),
                    district_type: Some(db::DistrictType::UsCongressional),
                    political_scope: db::PoliticalScope::Federal,
                    election_scope: db::ElectionScope::District,
                },
            ),
            (
                r"(?i:State Senat(?:e|or))",
                OfficeMeta {
                    name: "State Senate".into(),
                    title: "State Senator".into(),
                    chamber: Some(db::Chamber::Senate),
                    district_type: Some(db::DistrictType::StateHouse),
                    political_scope: db::PoliticalScope::State,
                    election_scope: db::ElectionScope::District,
                },
            ),
            (
                r"(?i:State (?:House|Representative))",
                OfficeMeta {
                    name: "State House".into(),
                    title: "State Representative".into(),
                    chamber: Some(db::Chamber::House),
                    district_type: Some(db::DistrictType::StateHouse),
                    political_scope: db::PoliticalScope::State,
                    election_scope: db::ElectionScope::District,
                },
            ),
            (
                r"(?i:Board of Education)",
                OfficeMeta {
                    name: "Board of Education".into(),
                    title: "Board of Education Member".into(),
                    chamber: None,
                    district_type: Some(db::DistrictType::UsCongressional),
                    political_scope: db::PoliticalScope::Local,
                    election_scope: db::ElectionScope::District,
                },
            ),
            (
                r"(?i:(?:Board of Regents|Regent of (?:the )?University))",
                OfficeMeta {
                    name: "Board of Regents".into(),
                    title: "Regent".into(),
                    chamber: None,
                    district_type: Some(db::DistrictType::UsCongressional),
                    political_scope: db::PoliticalScope::Local,
                    election_scope: db::ElectionScope::District,
                },
            ),
            (
                r"(?i:District Attorney)",
                OfficeMeta {
                    name: "District Attorney".into(),
                    title: "District Attorney".into(),
                    chamber: None,
                    district_type: Some(db::DistrictType::Judicial),
                    political_scope: db::PoliticalScope::Local,
                    election_scope: db::ElectionScope::District,
                },
            ),
            (
                r"(?i:District Court Judge)",
                OfficeMeta {
                    name: "District Court Judge".into(),
                    title: "District Court Judge".into(),
                    chamber: None,
                    district_type: Some(db::DistrictType::Judicial),
                    political_scope: db::PoliticalScope::Local,
                    election_scope: db::ElectionScope::District,
                },
            ),
            (
                r"(?i:County Court Judge)",
                OfficeMeta {
                    name: "County Court Judge".into(),
                    title: "County Court Judge".into(),
                    chamber: None,
                    district_type: None,
                    political_scope: db::PoliticalScope::Local,
                    election_scope: db::ElectionScope::County,
                },
            ),
            (
                r"(?i:Court of Appeals Judge)",
                OfficeMeta {
                    name: "Court of Appeals Judge".into(),
                    title: "Court of Appeals Judge".into(),
                    chamber: None,
                    district_type: None,
                    political_scope: db::PoliticalScope::State,
                    election_scope: db::ElectionScope::State,
                },
            ),
            (
                r"(?i:Supreme Court Justice)",
                OfficeMeta {
                    name: "Supreme Court Justice".into(),
                    title: "Supreme Court Justice".into(),
                    chamber: None,
                    district_type: None,
                    political_scope: db::PoliticalScope::State,
                    election_scope: db::ElectionScope::State,
                },
            ),
            (
                r"(?i:Regional Transportation District Director)",
                OfficeMeta {
                    name: "Regional Transportation District Director".into(),
                    title: "Regional Transportation District Director".into(),
                    chamber: None,
                    district_type: Some(db::DistrictType::Transportation),
                    political_scope: db::PoliticalScope::State,
                    election_scope: db::ElectionScope::District,
                },
            ),
        ]
        .into_iter()
        .map(|t| (Regex::new(t.0).unwrap(), t.1))
        .collect()
    });

    for (matcher, meta) in matchers {
        if matcher.is_match(input) {
            return Some(meta.clone());
        }
    }
    None
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
        if let Some(district) = extractor.captures(input).map(default_capture).flatten() {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_meta() {
        let tests: Vec<(&'static str, Option<&'static str>)> = vec![
            ("United States", None),
            ("U.S.", None),
            ("State", None),
            ("Senator", None),
            ("House", None),
            ("Representative", None),
            ("District", None),
            ("County", None),
            ("Board", None),
            ("Education", None),
            ("Regent", None),
            ("University", None),
            ("Attorney", None),
            ("Court", None),
            ("Judge", None),
            ("Appeals", None),
            ("Transportation", None),
            ("Director", None),
            // ----
            ("U.S. Senate", Some("U.S. Senate")),
            ("U.S. Senator", Some("U.S. Senate")),
            ("United States Senate", Some("U.S. Senate")),
            ("United States Senator", Some("U.S. Senate")),
            ("U.S. House", Some("U.S. House")),
            ("U.S. Representative", Some("U.S. House")),
            ("United States House", Some("U.S. House")),
            ("United States Representative", Some("U.S. House")),
            ("State Senate", Some("State Senate")),
            ("State Senator", Some("State Senate")),
            ("State House", Some("State House")),
            ("State Representative", Some("State House")),
            ("Board of Education", Some("Board of Education")),
            ("Board of Regents", Some("Board of Regents")),
            ("Regent of the University", Some("Board of Regents")),
            ("District Attorney", Some("District Attorney")),
            ("District Court Judge", Some("District Court Judge")),
            ("County Court Judge", Some("County Court Judge")),
            ("Court of Appeals Judge", Some("Court of Appeals Judge")),
            ("Supreme Court Justice", Some("Supreme Court Justice")),
            (
                "Regional Transportation District Director",
                Some("Regional Transportation District Director"),
            ),
        ];

        for (input, expected) in tests {
            assert_eq!(
                extract_office_meta(input)
                    .as_ref()
                    .map(|meta| meta.name.as_str()),
                expected,
                "\n\n  Test Case: '{input}'\n"
            );
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
}
