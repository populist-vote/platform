use std::{error::Error, fs::File, io::Read, path::PathBuf, sync::OnceLock};

use calamine::{RangeDeserializerBuilder, Reader, Xlsx};
use db::{BallotMeasureStatus, UpsertBallotMeasureInput};
use project_root::get_project_root;
use regex::Regex;
use serde::Deserialize;
use slugify::slugify;

use crate::generators::*;
use std::io::Cursor;

const FILE_PATH: &str = "mn/sos/ballot_measures.xlsx";
const PAGE_URL: &str =
    "https://www.sos.mn.gov/media/6162/questions-on-2024-state-general-election-ballot.xlsx";
const SOURCE_ID: &str = "MN-SOS";

#[derive(Default)]
pub struct Scraper {}

impl crate::Scraper for Scraper {
    fn source_id(&self) -> &'static str {
        SOURCE_ID
    }

    // Run for remote sources (byte array)
    async fn run(&self, context: &crate::ScraperContext<'_>) -> Result<(), Box<dyn Error>> {
        // Fetch the XLSX file as bytes from a remote source
        let bytes = reqwest::get(PAGE_URL).await?.bytes().await?.to_vec();
        let xlsx_source = XlsxSource::Bytes(bytes);

        // Open and scrape the XLSX file
        let xlsx = xlsx_source.open_xlsx()?;
        Self::scrape_xlsx(xlsx, context).await?;
        Ok(())
    }

    // Run for local sources (file path)
    async fn run_local(&self, context: &crate::ScraperContext<'_>) -> Result<(), Box<dyn Error>> {
        let path = get_project_root()?.join("scrapers/html").join(FILE_PATH);
        let xlsx_source = XlsxSource::Path(path.into());

        // Open and scrape the XLSX file
        let xlsx = xlsx_source.open_xlsx()?;
        Self::scrape_xlsx(xlsx, context).await?;
        Ok(())
    }
}

enum XlsxSource {
    Bytes(Vec<u8>),
    Path(PathBuf),
}

impl XlsxSource {
    // Open the XLSX file from either a byte array or a file path
    fn open_xlsx(self) -> Result<Xlsx<Cursor<Vec<u8>>>, Box<dyn Error>> {
        match self {
            XlsxSource::Bytes(bytes) => {
                // Create a Cursor for the in-memory bytes
                let cursor = Cursor::new(bytes);
                let xlsx = Xlsx::new(cursor)?;
                Ok(xlsx)
            }
            XlsxSource::Path(path) => {
                // Open the file and read its contents into a Vec<u8>
                let mut file = File::open(path)?;
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;

                // Create a Cursor from the byte buffer
                let cursor = Cursor::new(buffer);
                let xlsx = Xlsx::new(cursor)?;
                Ok(xlsx)
            }
        }
    }
}

impl Scraper {
    pub async fn scrape_xlsx(
        mut xlsx: Xlsx<impl std::io::Read + std::io::Seek>,
        context: &crate::ScraperContext<'_>,
    ) -> Result<(), Box<dyn Error>> {
        let election_year = Self::parse_election_year("2024 State General Election")?;
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

        let range = xlsx
            .with_header_row(calamine::HeaderRow::Row(1))
            .worksheet_range_at(0)
            .unwrap()
            .unwrap();

        let iter = RangeDeserializerBuilder::new().from_range(&range)?;

        for result in iter {
            // TODO: Figure out how to deserialize this as a struct, not a tuple
            let (
                county_id,
                fips_code,
                school_district_code,
                ballot_question_number,
                ballot_question_title,
                ballot_question_body,
            ): (String, String, String, String, String, String) = result?;

            if ballot_question_number.is_empty()
                || ballot_question_title.is_empty()
                || ballot_question_title == "no data"
            {
                continue;
            }

            let slug = slugify!(&format!(
                "mn-2024-{}-{}", // TODO: make year dynamic
                ballot_question_number.to_lowercase(),
                county_id
            ));

            let county_data = sqlx::query!(
                r#"SELECT countyname, countyfips FROM p6t_state_mn.bdry_votingdistricts WHERE countycode = $1"#,
                county_id
            )
            .fetch_optional(&context.db.connection)
            .await?;

            let (county, county_fips) = match county_data {
                Some(row) => (row.countyname, row.countyfips),
                None => (None, None),
            };

            let school_district = match school_district_code.as_str() {
                "no data" => None,
                _ => Some(school_district_code),
            };

            let municipality_fips = match fips_code.as_str() {
                "no data" => None,
                _ => Some(fips_code),
            };

            let ballot_measure_code = format!(
                "{} {}",
                ballot_question_number,
                match county.clone() {
                    Some(county) => county,
                    None => "".to_string(),
                }
            );

            let input = UpsertBallotMeasureInput {
                id: None,
                slug: Some(slug),
                election_id: Some(election.id),
                title: Some(ballot_question_title),
                status: Some(BallotMeasureStatus::InConsideration),
                state: Some(db::State::MN),
                ballot_measure_code: Some(ballot_measure_code),
                definitions: None,
                official_summary: None,
                populist_summary: None,
                full_text_url: None,
                description: Some(ballot_question_body),
                measure_type: None,
                county,
                municipality: None,
                school_district,
                county_fips,
                municipality_fips,
            };

            let _ballot_measure =
                db::BallotMeasure::upsert_from_source(&context.db.connection, &input)
                    .await
                    .expect("Error upserting ballot measure");
        }
        Ok(())
    }

    pub fn parse_election_year(title: &str) -> Result<u16, Box<dyn Error>> {
        static REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = REGEX.get_or_init(|| Regex::new(r"(\d{4}) State General Election").unwrap());
        let year = regex
            .captures(title)
            .ok_or("Unexpected election title format")?
            .get(1)
            .ok_or("Failure extracting election year from title")?
            .as_str()
            .parse::<u16>()?;
        Ok(year)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BallotMeasureEntry {
    pub county_id: String,
    pub fips_code: Option<String>,
    pub school_district_code: Option<String>,
    pub ballot_question_number: String,
    pub ballot_question_title: String,
    pub ballot_question_body: String,
}
