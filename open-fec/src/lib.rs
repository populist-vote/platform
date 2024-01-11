mod api;
pub use api::*;
use reqwest::{Error, Response};
use serde::Serialize;
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
    /// Office endpoint methods.
    pub const fn candidate(&self) -> Candidate<'_> {
        Candidate(self)
    }

    pub const fn candidates(&self) -> Candidates<'_> {
        Candidates(self)
    }
}
