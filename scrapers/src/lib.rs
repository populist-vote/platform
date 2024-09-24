use std::{error::Error, future::Future};

use slugify::slugify;

pub mod extractors;
pub mod mn_sos_candidate_filings_fed_state_county;
pub mod mn_sos_candidate_filings_local;
pub mod mn_sos_results;
pub mod util;

mod scrapers;

pub use scrapers::*;

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

pub fn generate_office_slug(input: &db::UpsertOfficeInput) -> String {
    slugify!(&format!(
        "{} {} {}",
        input.state.as_ref().map(|s| s.as_ref()).unwrap_or(""),
        input
            .name
            .as_ref()
            .map(|n| n.replace(".", ""))
            .as_ref()
            .map(String::as_str)
            .unwrap_or(""),
        match input.election_scope {
            Some(db::ElectionScope::District) => input
                .district
                .as_ref()
                .or(input.seat.as_ref())
                .map(String::as_str)
                .unwrap_or(""),
            Some(db::ElectionScope::County) =>
                input.county.as_ref().map(String::as_str).unwrap_or(""),
            _ => input.seat.as_ref().map(String::as_str).unwrap_or(""),
        }
    ))
}

#[cfg(test)]
mod tests {
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
