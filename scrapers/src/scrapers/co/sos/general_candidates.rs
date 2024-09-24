use std::error::Error;

use scraper::{Html, Selector};

use crate::{extractors::*, util, util::extensions::NoneIfEmptyExt};

const HTML_PATH: &'static str = "co/sos/general_candidates.html";
const PAGE_URL: &'static str =
    "https://www.sos.state.co.us/pubs/elections/vote/generalCandidates.html";

#[derive(Default)]
pub struct Scraper {}

impl crate::Scraper for Scraper {
    async fn run(&self, context: &crate::ScraperContext<'_>) -> Result<(), Box<dyn Error>> {
        let html = reqwest::get(PAGE_URL).await?.text().await?;
        Self::scrape_html(html, context).await
    }

    async fn run_local(&self, context: &crate::ScraperContext<'_>) -> Result<(), Box<dyn Error>> {
        let html = util::read_local_html(&HTML_PATH)?;
        Self::scrape_html(html, context).await
    }
}

impl Scraper {
    pub async fn scrape_html(
        html: String,
        context: &crate::ScraperContext<'_>,
    ) -> Result<(), Box<dyn Error>> {
        let entries = Self::parse_raw_entries(html)?;
        for entry in entries {
            let mut office = Self::build_office_input(&entry);
            office.slug = Some(crate::generate_office_slug(&office));
            if let Err(err) = db::Office::upsert_from_source(&context.db.connection, &office).await
            {
                // TODO - Track/log error
                panic!("{err}");
            }
        }
        Ok(())
    }

    pub fn parse_raw_entries(html: String) -> Result<Vec<RawEntry>, Box<dyn Error>> {
        let mut entries = Vec::new();
        let document = Html::parse_document(&html);
        for (index, element) in document
            .select(&Selector::parse("table.w3-cmsTable")?)
            .next()
            .ok_or("Candidate table not found")?
            .select(&Selector::parse("tbody")?)
            .next()
            .ok_or("Candidate table body not found")?
            .select(&Selector::parse("tr")?)
            .enumerate()
        {
            let selector = Selector::parse("td")?;
            let mut fields = element
                .select(&selector)
                .map(|td| td.text().collect::<String>());

            let mut next_field = |label: &'static str| -> Result<Option<String>, Box<dyn Error>> {
                fields
                    .next()
                    .ok_or_else(|| format!("Missing {label} field"))
                    .map_err(|err| err.into())
                    .map(|f| f.none_if_empty())
            };

            let entry = RawEntry {
                index,
                name: next_field("name")?.ok_or("Empty name field")?,
                office: next_field("office")?.ok_or("Empty office field")?,
                party: next_field("party")?,
                _write_in: next_field("write_in")?,
                website: next_field("website")?,
            };

            if !entry.office.starts_with("President") {
                entries.push(entry);
            }
        }
        Ok(entries)
    }

    fn build_office_input(entry: &RawEntry) -> db::UpsertOfficeInput {
        let Some(meta) = extract_office_meta(&entry.office) else {
            // TODO - Track/log failed scrape
            return db::UpsertOfficeInput::default();
        };

        let (district, seat) = if meta.election_scope == db::ElectionScope::District {
            if let Some(qualifier) = extract_office_qualifier(&entry.office) {
                match qualifier {
                    OfficeQualifier::District(district) => (Some(district.clone()), Some(district)),
                    OfficeQualifier::AtLarge => (None, Some(qualifier.as_ref().to_string())),
                }
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        let county = if meta.election_scope == db::ElectionScope::County {
            entry.office.split(" - ").last().map(str::to_string)
        } else {
            None
        };

        return db::UpsertOfficeInput {
            name: Some(meta.name),
            title: Some(meta.title),
            chamber: meta.chamber,
            seat,
            district,
            district_type: meta.district_type,
            county,
            state: Some(db::State::CO),
            political_scope: Some(meta.political_scope),
            election_scope: Some(meta.election_scope),
            ..Default::default()
        };
    }
}

pub struct RawEntry {
    pub index: usize,
    pub name: String,
    pub office: String,
    pub party: Option<String>,
    pub website: Option<String>,
    _write_in: Option<String>,
}
