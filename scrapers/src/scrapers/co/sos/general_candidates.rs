use std::{error::Error, sync::OnceLock};

use regex::Regex;
use scraper::{Html, Selector};

use crate::{
    extractors::*,
    generators::{
        ElectionTitleGenerator, GeneralElectionDateGenerator, OfficeSlugGenerator,
        RaceTitleGenerator,
    },
    util::{self, extensions::NoneIfEmptyExt},
};

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
        let data = Self::scrape_page_data(html)?;

        let election_year = Self::parse_election_year(&data.title)?;
        let election_date = GeneralElectionDateGenerator::new(election_year).generate()?;
        let (election_title, election_slug) =
            ElectionTitleGenerator::new(&db::RaceType::General, election_year).generate();
        let election = db::Election::upsert_from_source(
            &context.db.connection,
            &db::UpsertElectionInput {
                slug: Some(election_slug),
                title: Some(election_title),
                election_date: Some(election_date),
                ..Default::default()
            },
        )
        .await?;

        for entry in data.candidates {
            let office = Self::build_office_input(&entry);
            let office = match db::Office::upsert_from_source(&context.db.connection, &office).await
            {
                Ok(office) => office,
                Err(err) => {
                    // TODO - Track/log error
                    println!("Error upserting office {err}");
                    continue;
                }
            };

            let race = Self::build_race_input(&election, &office);
            let _race = match db::Race::upsert_from_source(&context.db.connection, &race).await {
                Ok(race) => race,
                Err(err) => {
                    // TODO - Track/log error
                    println!("Error upserting race {err}");
                    continue;
                }
            };
        }
        Ok(())
    }

    pub fn scrape_page_data(html: String) -> Result<PageData, Box<dyn Error>> {
        let html = Html::parse_document(&html);
        Ok(PageData {
            title: Self::scrape_page_title(&html)?,
            candidates: Self::scrape_candidate_entries(&html)?,
        })
    }

    pub fn scrape_page_title(html: &Html) -> Result<String, Box<dyn Error>> {
        let title = html
            .select(&Selector::parse("p.pageTitle")?)
            .next()
            .ok_or("Page title not found")?
            .text()
            .collect::<String>();
        Ok(title)
    }

    pub fn scrape_candidate_entries(html: &Html) -> Result<Vec<CandidateEntry>, Box<dyn Error>> {
        let mut entries = Vec::new();
        for (index, element) in html
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

            let entry = CandidateEntry {
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

    pub fn parse_election_year(title: &str) -> Result<u16, Box<dyn Error>> {
        static REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = REGEX.get_or_init(|| Regex::new(r"(\d{4}) General Election").unwrap());
        let year = regex
            .captures(title)
            .ok_or("Unexpected election title format")?
            .get(1)
            .ok_or("Failure extracting election year from title")?
            .as_str()
            .parse::<u16>()?;
        Ok(year)
    }

    fn build_office_input(entry: &CandidateEntry) -> db::UpsertOfficeInput {
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

        let mut office = db::UpsertOfficeInput {
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
        office.slug = Some(OfficeSlugGenerator::from_source(&office).generate());
        return office;
    }

    fn build_race_input(election: &db::Election, office: &db::Office) -> db::UpsertRaceInput {
        let (title, slug) =
            RaceTitleGenerator::from_source(&db::RaceType::General, election, office).generate();
        db::UpsertRaceInput {
            title: Some(title),
            slug: Some(slug),
            office_id: Some(office.id),
            election_id: Some(election.id),
            state: Some(db::State::CO),
            race_type: Some(db::RaceType::General),
            vote_type: Some(db::VoteType::Plurality),
            ..Default::default()
        }
    }
}

pub struct PageData {
    pub title: String,
    pub candidates: Vec<CandidateEntry>,
}

pub struct CandidateEntry {
    pub index: usize,
    pub name: String,
    pub office: String,
    pub party: Option<String>,
    pub website: Option<String>,
    _write_in: Option<String>,
}
