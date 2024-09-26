use std::{error::Error, future::Future};

pub mod extractors;
pub mod generators;
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
