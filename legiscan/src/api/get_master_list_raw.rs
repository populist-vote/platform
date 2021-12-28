use crate::{Error, LegiscanProxy};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct GetMasterListRawResponse {
    status: String,
    masterlist: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
pub struct MasterListRawBill {
    bill_id: i32,
    number: String,
    change_hash: String,
}

impl LegiscanProxy {
    /// Retrieve master bill list optimized for change_hash detection
    /// List of bill information including bill_id and bill_number. The change_hash is a representation of the current bill
    /// status, it should be stored for a quick comparison to subsequent getMasterListRaw calls to detect what bills have
    /// changed and need updating.
    // 1 hour
    pub async fn get_master_list_raw_by_state(
        &self,
        state: &str,
    ) -> Result<Vec<MasterListRawBill>, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&state={state}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getMasterListRaw",
            state = state
        );
        let response = self.client.get(url).send().await.unwrap();

        match crate::handle_legiscan_response(response).await {
            Ok(json) => {
                let masterlist: Vec<MasterListRawBill> = json["masterlist"]
                    .as_object()
                    .unwrap()
                    .iter()
                    .filter(|(key, _val)| key.parse::<i32>().is_ok())
                    .map(|(_key, value)| serde_json::from_value(value.to_owned()).unwrap())
                    .collect();
                Ok(masterlist)
            }
            Err(e) => Err(e),
        }
    }

    /// Retrieve master bill list optimized for change_hash detection
    /// List of bill information including bill_id and bill_number. The change_hash is a representation of the current bill
    /// status, it should be stored for a quick comparison to subsequent getMasterListRaw calls to detect what bills have
    /// changed and need updating.
    // 1 hour
    pub async fn get_master_list_raw_by_session(
        &self,
        session_id: i32,
    ) -> Result<Vec<MasterListRawBill>, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&id={session_id}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getMasterListRaw",
            session_id = session_id
        );
        println!("{}", url);
        let response = self.client.get(url).send().await.unwrap();

        match crate::handle_legiscan_response(response).await {
            Ok(json) => {
                let masterlist: Vec<MasterListRawBill> = json["masterlist"]
                    .as_object()
                    .unwrap()
                    .iter()
                    .filter(|(key, _val)| key.parse::<i32>().is_ok())
                    .map(|(_key, value)| serde_json::from_value(value.to_owned()).unwrap())
                    .collect();
                Ok(masterlist)
            }
            Err(e) => Err(e),
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_get_master_list_raw_by_state() {
    let proxy = LegiscanProxy::new().unwrap();
    let masterlist = proxy.get_master_list_raw_by_state("CO").await.unwrap();
    assert_eq!(masterlist.len() >= 678, true);
}

#[tokio::test]
#[ignore]
async fn test_get_master_list_raw_by_session() {
    let proxy = LegiscanProxy::new().unwrap();
    let masterlist = proxy.get_master_list_raw_by_session(1797).await.unwrap();
    assert_eq!(masterlist.len() >= 678, true);
}
