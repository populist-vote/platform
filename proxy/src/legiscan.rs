use crate::errors::Error;
use serde::{Deserialize, Serialize};

const LEGISCAN_BASE_URL: &str = "https://api.legiscan.com/";

/// Struct used to make calls to Legiscan API
#[derive(Debug, Clone)]
pub struct LegiscanProxy {
    client: reqwest::Client,
    pub base_url: reqwest::Url,
    api_key: String,
}

#[derive(Serialize, Deserialize)]
struct GetBillResponse {
    status: String,
    bill: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct GetBillTextResponse {
    status: String,
    bill: serde_json::Value,
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

    pub async fn get_bill(&self, bill_id: String) -> Result<serde_json::Value, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&id={bill_id}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getBill",
            bill_id = bill_id
        );
        let response = self.client.get(url).send().await.unwrap();
        let json: GetBillResponse = response.json().await?;
        let bill_data = json.bill;
        Ok(bill_data)
    }

    pub async fn get_bill_text(&self, bill_id: String) -> Result<serde_json::Value, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&id={bill_id}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getBillText",
            bill_id = bill_id
        );
        let response = self.client.get(url).send().await.unwrap();
        let json: GetBillTextResponse = response.json().await?;
        let bill_text_data = json.bill;
        Ok(bill_text_data)
    }
}
