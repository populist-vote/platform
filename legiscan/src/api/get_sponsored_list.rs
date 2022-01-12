use crate::{Error, LegiscanProxy};
use serde::{Deserialize, Serialize};

use super::Person;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetSponsoredListResponse {
    pub status: String,
    #[serde(rename = "sponsoredbills")]
    pub sponsored_bills: SponsoredBills,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SponsoredBills {
    pub sponsor: Person,
    pub sessions: Vec<Session>,
    pub bills: Vec<BillInfo>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub session_id: i32,
    pub session_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillInfo {
    pub session_id: i32,
    pub bill_id: i32,
    pub number: String,
}

impl LegiscanProxy {
    /// Retrieve list of bills sponsored by an individual people_id
    // daily
    pub async fn get_sponsored_list(&self, people_id: i32) -> Result<SponsoredBills, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&id={people_id}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getSponsoredList",
            people_id = people_id
        );
        let response = self.client.get(url).send().await.unwrap();
        match crate::handle_legiscan_response(response).await {
            Ok(json) => {
                let json: GetSponsoredListResponse = serde_json::from_value(json).unwrap();
                Ok(json.sponsored_bills)
            }
            Err(e) => Err(e),
        }
    }
}

#[tokio::test]
async fn test_get_sponsored_list() {
    let proxy = LegiscanProxy::new().unwrap();
    let sponsored_bills = proxy.get_sponsored_list(1498).await.unwrap();
    assert_eq!(sponsored_bills.sponsor.name, "Jim Beall");
}
