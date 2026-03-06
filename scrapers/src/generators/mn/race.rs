use std::sync::OnceLock;

use regex::Regex;
use slugify::slugify;

use crate::util::extensions::*;

pub struct RaceTitleGenerator<'a> {
    pub race_type: &'a db::RaceType,
    pub election_scope: &'a db::ElectionScope,
    pub office_name: Option<&'a str>,
    pub office_subtitle: Option<&'a str>,
    pub state: Option<&'a db::State>,
    pub county: Option<&'a str>,
    pub district: Option<&'a str>,
    pub seat: Option<&'a str>,
    pub is_special_election: bool,
    pub party: Option<&'a str>,
    pub year: i32,
}

impl<'a> RaceTitleGenerator<'a> {
    pub fn from_source(
        r#type: &'a db::RaceType,
        office: &'a db::Office,
        is_special_election: bool,
        party: Option<&'a str>,
        year: i32,
    ) -> Self {
        RaceTitleGenerator {
            race_type: r#type,
            election_scope: &office.election_scope,
            office_name: office.name.as_deref(),
            office_subtitle: office.subtitle.as_deref(),
            state: office.state.as_ref(),
            county: office.county.as_deref(),
            district: office.district.as_deref(),
            seat: office.seat.as_deref(),
            is_special_election,
            party,
            year,
        }
    }

    pub fn generate(&self) -> (String, String) {
        // Build title following dbt logic: MN - <office_name> - <subtitle> - [Special Election -] <race_type> [- party] - <year>
        let mut parts = Vec::new();

        // Always start with MN
        parts.push("MN".to_string());

        // Office name
        if let Some(name) = self.office_name {
            if !name.is_empty() {
                parts.push(name.to_string());
            }
        }

        // Office subtitle (already contains location info, may have "MN - " prefix)
        if let Some(subtitle) = self.office_subtitle {
            if !subtitle.is_empty() {
                // Remove "MN - " prefix if it exists to avoid duplication
                let cleaned_subtitle = if subtitle.starts_with("MN - ") {
                    &subtitle[5..] // Skip "MN - "
                } else {
                    subtitle
                };

                // Only add if there's content after cleaning
                if !cleaned_subtitle.is_empty() {
                    parts.push(cleaned_subtitle.to_string());
                }
            }
        }

        // Special Election (before race type in SQL)
        if self.is_special_election {
            parts.push("Special Election".to_string());
        }

        // Race type with party expansion for primaries
        let race_type_str = match self.race_type {
            db::RaceType::Primary => match self.party {
                Some("N") => "Primary - Nonpartisan".to_string(),
                Some("REP") => "Primary - Republican".to_string(),
                Some("DEM") | Some("DFL") => "Primary - Democratic".to_string(),
                Some(p) if !p.is_empty() => format!("Primary - {}", p),
                _ => "Primary".to_string(),
            },
            db::RaceType::General => "General".to_string(),
        };
        parts.push(race_type_str);

        // Year
        parts.push(self.year.to_string());

        // Join with " - " and clean up extra spaces
        let title = parts.join(" - ");

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
