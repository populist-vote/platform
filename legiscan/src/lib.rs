mod api;
pub use api::*;
#[allow(clippy::module_inception)]
mod errors;
use errors::{Error, LegiscanErrorResponse};

const LEGISCAN_BASE_URL: &str = "https://api.legiscan.com/";

/// Struct used to make calls to Legiscan API
#[derive(Debug, Clone)]
pub struct LegiscanProxy {
    client: reqwest::Client,
    pub base_url: reqwest::Url,
    api_key: String,
}

impl LegiscanProxy {
    pub fn new() -> Result<Self, Error> {
        dotenv::dotenv().ok();
        let api_key = std::env::var("LEGISCAN_API_KEY")?;
        let client = reqwest::Client::new();

        Ok(LegiscanProxy {
            client,
            base_url: reqwest::Url::parse(LEGISCAN_BASE_URL).unwrap(),
            api_key,
        })
    }

    /// Instantiate new LegiscanProxy API client by passing api key to this function
    pub fn new_from_key(api_key: String) -> Result<Self, Error> {
        let client = reqwest::Client::new();

        Ok(LegiscanProxy {
            client,
            base_url: reqwest::Url::parse(LEGISCAN_BASE_URL).unwrap(),
            api_key,
        })
    }
}

pub async fn handle_legiscan_response(
    response: reqwest::Response,
) -> Result<serde_json::Value, Error> {
    if response.status().is_success() {
        let json: serde_json::Value = response.json().await?;
        match json["status"].as_str().unwrap() {
            "OK" => Ok(json),
            "ERROR" => {
                let json: LegiscanErrorResponse = serde_json::from_value(json).unwrap();
                Err(Error::Api(json.alert.message))
            }
            _ => Err(Error::Api("Something went wrong.".to_string())),
        }
    } else {
        Err(Error::Api("Legiscan API could not be reached.".to_string()))
    }
}
