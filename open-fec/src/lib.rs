mod types;
use reqwest::{Error, Response};
use serde::Serialize;
use types::candidate::{CandidateQuery, CandidatesQuery};
pub use types::*;
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
            .query(&[("api_key", &self.api_key)]);

        let res = res.send().await?;
        Ok(res)
    }
}

/// Endpoint function namespaces.
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
