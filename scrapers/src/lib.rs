use std::{error::Error, future::Future};

pub mod extractors;
pub mod generators;
pub mod util;
pub mod processors;

mod scrapers;

pub use scrapers::*;

pub struct ScraperContext<'a> {
    pub db: &'a db::DatabasePool,
}

pub trait Scraper {
    fn source_id(&self) -> &'static str;
    fn run(
        &self,
        context: &ScraperContext,
    ) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
    fn run_local(
        &self,
        context: &ScraperContext,
    ) -> impl Future<Output = Result<(), Box<dyn Error>>> + Send;
}
