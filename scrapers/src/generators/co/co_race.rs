use std::sync::OnceLock;

use chrono::Datelike;
use regex::Regex;
use slugify::slugify;

use crate::util::extensions::*;
use crate::generators::optional_state_str;

pub struct RaceTitleGenerator<'a> {
    pub race_type: &'a db::RaceType,
    pub election_scope: &'a db::ElectionScope,
    pub office_name: Option<&'a str>,
    pub state: Option<&'a db::State>,
    pub county: Option<&'a str>,
    pub district: Option<&'a str>,
    pub seat: Option<&'a str>,
    pub year: i32,
}

impl<'a> RaceTitleGenerator<'a> {
    pub fn from_source(
        r#type: &'a db::RaceType,
        election: &'a db::Election,
        office: &'a db::Office,
    ) -> Self {
        RaceTitleGenerator {
            race_type: r#type,
            election_scope: &office.election_scope,
            office_name: office.name.as_str(),
            state: office.state.as_ref(),
            county: office.county.as_str(),
            district: office.district.as_str(),
            seat: office.seat.as_str(),
            year: election.election_date.year(),
        }
    }

    pub fn generate(&self) -> (String, String) {
        let qualifier = match self.election_scope {
            db::ElectionScope::County => (self.county.unwrap_or_default(), "County"),
            _ => {
                if self.district.is_some() {
                    ("District", self.district.unwrap_or_default())
                } else {
                    ("", self.seat.unwrap_or_default())
                }
            }
        };

        let title = format!(
            "{} {} {} {} {} {}",
            optional_state_str(self.state),
            self.office_name.unwrap_or_default(),
            qualifier.0,
            qualifier.1,
            self.race_type,
            self.year,
        );

        static REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = REGEX.get_or_init(|| Regex::new(r"  +").unwrap());
        let title = regex.replace_all(&title, " ").trim().to_string();
        let slug = slugify!(&title.replace(".", ""));
        (title, slug)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn race_title() {
        let tests: Vec<((&'static str, &'static str), RaceTitleGenerator)> = vec![
            (
                (
                    "CO Supreme Court Justice General 2024",
                    "co-supreme-court-justice-general-2024",
                ),
                RaceTitleGenerator {
                    race_type: &db::RaceType::General,
                    election_scope: &db::ElectionScope::State,
                    office_name: Some("Supreme Court Justice"),
                    state: Some(&db::State::CO),
                    county: None,
                    district: None,
                    seat: None,
                    year: 2024,
                },
            ),
            (
                (
                    "CO U.S. House District 1 General 2024",
                    "co-us-house-district-1-general-2024",
                ),
                RaceTitleGenerator {
                    race_type: &db::RaceType::General,
                    election_scope: &db::ElectionScope::State,
                    office_name: Some("U.S. House"),
                    state: Some(&db::State::CO),
                    county: None,
                    district: Some("1"),
                    seat: None,
                    year: 2024,
                },
            ),
            (
                (
                    "CO Board of Regents At Large General 2024",
                    "co-board-of-regents-at-large-general-2024",
                ),
                RaceTitleGenerator {
                    race_type: &db::RaceType::General,
                    election_scope: &db::ElectionScope::State,
                    office_name: Some("Board of Regents"),
                    state: Some(&db::State::CO),
                    county: None,
                    district: None,
                    seat: Some("At Large"),
                    year: 2024,
                },
            ),
            (
                (
                    "CO County Court Judge Adams County General 2024",
                    "co-county-court-judge-adams-county-general-2024",
                ),
                RaceTitleGenerator {
                    race_type: &db::RaceType::General,
                    election_scope: &db::ElectionScope::County,
                    office_name: Some("County Court Judge"),
                    state: Some(&db::State::CO),
                    county: Some("Adams"),
                    district: None,
                    seat: None,
                    year: 2024,
                },
            ),
        ];

        for (expected, generator) in tests {
            let actual = generator.generate();
            assert_eq!(expected.0, actual.0);
            assert_eq!(expected.1, actual.1);
        }
    }
}
