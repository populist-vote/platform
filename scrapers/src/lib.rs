use std::{error::Error, future::Future};

use chrono::{Datelike, Days, NaiveDate, Weekday};
use slugify::slugify;

pub mod extractors;
pub mod mn_sos_candidate_filings_fed_state_county;
pub mod mn_sos_candidate_filings_local;
pub mod mn_sos_results;
pub mod util;

mod scrapers;

pub use scrapers::*;

use util::extensions::*;

pub struct ScraperContext<'a> {
    pub db: &'a db::DatabasePool,
}

pub trait Scraper {
    fn run(
        &self,
        context: &ScraperContext,
    ) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
    fn run_local(
        &self,
        context: &ScraperContext,
    ) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
}

// Reference: https://en.wikipedia.org/wiki/Election_Day_(United_States)
// "The Tuesday after the first Monday of November"
pub fn generate_general_election_date(year: u16) -> Result<NaiveDate, Box<dyn Error>> {
    let first_monday = NaiveDate::from_weekday_of_month_opt(year as _, 11, Weekday::Mon, 1)
        .ok_or_else(|| format!("Unable to determine general election date for year: {year}"))?;
    let next_tuesday = first_monday
        .checked_add_days(Days::new(1))
        .ok_or_else(|| format!("Unable to determine general election date for year: {year}"))?;
    Ok(next_tuesday)
}

pub fn generate_general_election_title_slug(year: u16) -> (String, String) {
    let title = format!("General Election {year}");
    let slug = slugify!(&title);
    (title, slug)
}

pub fn generate_race_title_slug(
    election: &db::Election,
    office: &db::Office,
    race_type: db::RaceType,
) -> (String, String) {
    let qualifier = match office.election_scope {
        db::ElectionScope::County => (office.county.as_str_unwrapped_or_empty(), " County"),
        _ => {
            if office.district.is_some() {
                ("District ", office.district.as_str_unwrapped_or_empty())
            } else {
                ("", office.seat.as_str_unwrapped_or_empty())
            }
        }
    };

    let title = format!(
        "{} {} {}{} {} {}",
        state_str(&office.state),
        office.name.as_str_unwrapped_or_empty(),
        qualifier.0,
        qualifier.1,
        race_type,
        election.election_date.year(),
    );

    let slug = slugify!(&title.replace(".", ""));
    (title, slug)
}

pub fn generate_office_slug(input: &db::UpsertOfficeInput) -> String {
    let format = format!(
        "{} {} {}",
        state_str(&input.state),
        input
            .name
            .as_ref()
            .map(|n| n.replace(".", ""))
            .as_str_unwrapped_or_empty(),
        match input.election_scope {
            Some(db::ElectionScope::County) => input.county.as_str_unwrapped_or_empty(),
            _ =>
                if input.district.is_some() {
                    input.district.as_str_unwrapped_or_empty()
                } else {
                    input.seat.as_str_unwrapped_or_empty()
                },
        }
    );
    slugify!(&format)
}

#[inline]
fn state_str<'a>(state: &'a Option<db::State>) -> &'a str {
    state.as_ref().map(|s| s.as_ref()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use chrono::Datelike;

    #[test]
    fn generate_general_election_date() {
        let tests: Vec<((u32, u32), u16)> = vec![
            ((11, 7), 2023),
            ((11, 5), 2024),
            ((11, 4), 2025),
            ((11, 3), 2026),
            ((11, 2), 2027),
            ((11, 7), 2028),
        ];

        for (expected, input) in tests {
            let date = super::generate_general_election_date(input).unwrap();
            assert_eq!(expected, (date.month(), date.day()));
        }
    }

    #[test]
    fn generate_general_election_title_slug() {
        let tests: Vec<((&'static str, &'static str), u16)> = vec![
            (("General Election 2024", "general-election-2024"), 2024),
            (("General Election 2025", "general-election-2025"), 2025),
        ];

        for (expected, input) in tests {
            let actual = super::generate_general_election_title_slug(input);
            assert_eq!(expected.0, actual.0);
            assert_eq!(expected.1, actual.1);
        }
    }

    #[test]
    fn generate_office_slug() {
        let tests: Vec<(&'static str, db::UpsertOfficeInput)> = vec![
            (
                "co-us-senate-1",
                db::UpsertOfficeInput {
                    state: Some(db::State::CO),
                    name: Some("U.S. Senate".into()),
                    seat: Some("1".into()),
                    ..Default::default()
                },
            ),
            (
                "co-us-house-1",
                db::UpsertOfficeInput {
                    state: Some(db::State::CO),
                    name: Some("U.S. House".into()),
                    election_scope: Some(db::ElectionScope::District),
                    district: Some("1".into()),
                    ..Default::default()
                },
            ),
            (
                "co-district-something-at-large",
                db::UpsertOfficeInput {
                    state: Some(db::State::CO),
                    name: Some("District Something".into()),
                    election_scope: Some(db::ElectionScope::District),
                    district: None,
                    seat: Some("At Large".into()),
                    ..Default::default()
                },
            ),
            (
                "co-district-something",
                db::UpsertOfficeInput {
                    state: Some(db::State::CO),
                    name: Some("District Something".into()),
                    election_scope: Some(db::ElectionScope::District),
                    // no district or seat specified
                    ..Default::default()
                },
            ),
            (
                "co-county-judge-adams",
                db::UpsertOfficeInput {
                    state: Some(db::State::CO),
                    name: Some("County Judge".into()),
                    election_scope: Some(db::ElectionScope::County),
                    county: Some("Adams".into()),
                    ..Default::default()
                },
            ),
            (
                "co-county-judge",
                db::UpsertOfficeInput {
                    state: Some(db::State::CO),
                    name: Some("County Judge".into()),
                    election_scope: Some(db::ElectionScope::County),
                    // no county specified
                    ..Default::default()
                },
            ),
            (
                "co-court-of-appeals-judge",
                db::UpsertOfficeInput {
                    state: Some(db::State::CO),
                    name: Some("Court of Appeals Judge".into()),
                    ..Default::default()
                },
            ),
        ];

        for (expected, input) in tests {
            assert_eq!(expected, super::generate_office_slug(&input));
        }
    }
}
