use reqwest::{Error, Response};
use serde::{Deserialize, Serialize};
const OPEN_FEC_BASE_URL: &str = "https://api.open.fec.gov";

pub struct OpenFecProxy {
    client: reqwest::Client,
    pub base_url: reqwest::Url,
    api_key: String,
}

impl OpenFecProxy {
    /// Instantiate new OpenFecProxy API client from API within .env file
    pub fn new() -> Result<Self, std::env::VarError> {
        dotenv::dotenv().ok();
        let api_key = std::env::var("OPEN_FEC_API_KEY")?;
        let client = reqwest::Client::new();

        Ok(OpenFecProxy {
            client,
            base_url: reqwest::Url::parse(OPEN_FEC_BASE_URL).unwrap(),
            api_key,
        })
    }

    pub fn new_from_key(api_key: String) -> Result<Self, Error> {
        let client = reqwest::Client::new();

        Ok(OpenFecProxy {
            client,
            base_url: reqwest::Url::parse(OPEN_FEC_BASE_URL).unwrap(),
            api_key,
        })
    }

    pub async fn get(&self, path: &str, query: impl Serialize) -> Result<Response, Error> {
        let url = self.base_url.join(path).unwrap();
        let res = self
            .client
            .get(url)
            .query(&query)
            .query(&[("api_key", &self.api_key)])
            .send()
            .await?;
        Ok(res)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct CandidateQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub cycle: Option<Vec<u32>>,
    pub election_year: Option<Vec<u32>>,
    pub office: Option<Vec<String>>,
    pub state: Option<Vec<String>>,
    pub party: Option<Vec<String>>,
    pub year: Option<String>,
    pub district: Option<Vec<String>>,
    pub candidate_status: Option<Vec<String>>,
    pub incumbent_challenge: Option<Vec<String>>,
    pub federal_funds_flag: Option<bool>,
    pub has_raised_funds: Option<bool>,
    pub name: Option<Vec<String>>,
    pub sort: Option<String>,
    pub sort_hide_null: Option<bool>,
    pub sort_null_only: Option<bool>,
    pub sort_nulls_last: Option<bool>,
    pub api_key: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct CandidatesQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub q: Option<String>,
    pub candidate_id: Option<Vec<String>>,
    pub min_first_file_date: Option<String>,
    pub max_first_file_date: Option<String>,
    pub is_active_candidate: Option<bool>,
    pub cycle: Option<Vec<u32>>,
    pub election_year: Option<Vec<u32>>,
    pub office: Option<String>,
    pub state: Option<String>,
    pub party: Option<Vec<String>>,
    pub year: Option<String>,
    pub district: Option<Vec<String>>,
    pub candidate_status: Option<Vec<String>>,
    pub incumbent_challenge: Option<Vec<String>>,
    pub federal_funds_flag: Option<bool>,
    pub has_raised_funds: Option<bool>,
    pub name: Option<Vec<String>>,
    pub sort: Option<String>,
    pub sort_hide_null: Option<bool>,
    pub sort_null_only: Option<bool>,
    pub sort_nulls_last: Option<bool>,
    pub api_key: Option<String>,
}

impl OpenFecProxy {
    pub async fn get_candidate(
        &self,
        candidate_id: &str,
        query: CandidateQuery,
    ) -> Result<Response, Error> {
        let path = format!("/v1/candidate/{}", candidate_id);
        self.get(&path, query).await
    }

    pub async fn get_candidates(&self, query: CandidatesQuery) -> Result<Response, Error> {
        let path = "/v1/candidates";
        self.get(&path, query).await
    }
}
