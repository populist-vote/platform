mod errors;
mod tests;
use errors::Error;
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

#[derive(Serialize, Deserialize)]
struct GetMasterListResponse {
    status: String,
    masterlist: serde_json::Value,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MasterListBill {
    pub bill_id: i64,
    pub number: String,
    pub change_hash: String,
    pub url: String,
    pub status_date: String,
    pub status: String,
    pub last_action_date: String,
    pub last_action: String,
    pub title: String,
    pub description: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetSessionListResponse {
    pub status: String,
    pub sessions: Vec<Session>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub session_id: i64,
    pub state_id: i64,
    pub year_start: i64,
    pub year_end: i64,
    pub special: i64,
    pub session_name: String,
    pub name: String,
    pub session_hash: String,
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

    /// Retrieve a list of available sessions for the given state abbreviation
    /// Refresh daily
    /// List of session information including session_id for subsequent getMasterList calls along with session years, special
    /// session indicator and the session_hash which reflects the current dataset version for the session_id for identifying
    /// and tracking when bills change.
    pub async fn get_session_list(&self, state: &str) -> Result<Vec<Session>, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&state={state}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getSessionList",
            state = state
        );
        let response = self.client.get(url).send().await.unwrap();
        let json: GetSessionListResponse = response.json().await?;
        let sessions_data = json.sessions;
        Ok(sessions_data)
    }

    /// This operation returns a master list of summary bill data in the given session_id or current state session.
    // 1 hour
    pub async fn get_master_list_by_state(
        &self,
        state: String,
    ) -> Result<serde_json::Value, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&state={state}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getMasterList",
            state = state
        );
        let response = self.client.get(url).send().await.unwrap();
        let json: GetMasterListResponse = response.json().await?;
        let masterlist_data = json.masterlist;
        Ok(masterlist_data)
    }

    /// Retrieve master bill list for a session
    // 1 hour
    pub async fn get_master_list_by_session(&self, session_id: i32) {
        todo!()
    }

    /// Retrieve master bill list optimized for change_hash detection
    // 1 hour
    pub async fn get_master_list_raw() {
        todo!()
    }

    // 3 hours
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

    // static
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

    /// Retrieve amendment text for a given amendment_id
    // static
    pub async fn get_amendment() {
        todo!()
    }

    /// Retrieve supplemental document for a given supplement_id
    // static
    pub async fn get_supplement() {
        todo!()
    }

    /// Retrieve roll call vote information for a given roll_call_id
    // static
    pub async fn get_roll_call() {
        todo!()
    }

    /// Retrieve basic information for a given people_id
    // weekly
    pub async fn get_person() {
        todo!()
    }

    /// Retrieve results from the full text search engine (50 results)
    // 1 hour
    pub async fn search() {
        todo!()
    }

    /// Retrieve results from the full text search engine (2000 results)
    // 1 hour
    pub async fn search_raw() {
        todo!()
    }

    /// Retrieve list of available dataset snapshots
    // weekly
    pub async fn get_dataset_list() {
        todo!()
    }

    /// Retrieve an individual dataset for a specific `session_id`
    // weekly
    pub async fn get_dataset() {
        todo!()
    }

    /// Retrieve list of people active in a specific `session_id`
    // weekly
    pub async fn get_session_people() {
        todo!()
    }

    /// Retrieve list of bills sponsored by an individual people_id
    // daily
    pub async fn get_sponsored_list() {
        todo!()
    }
}
