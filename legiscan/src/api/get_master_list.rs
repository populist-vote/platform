use crate::{Error, LegiscanProxy};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct GetMasterListResponse {
    status: String,
    masterlist: serde_json::Value,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MasterListBill {
    pub bill_id: i32,
    pub number: String,
    pub change_hash: String,
    pub url: String,
    pub status_date: Option<String>,
    pub status: serde_json::Value, // sometimes a string
    pub last_action_date: String,
    pub last_action: String,
    pub title: String,
    pub description: String,
}

impl LegiscanProxy {
    /// This operation returns a master list of summary bill data in the given session_id or current state session.
    // 1 hour
    pub async fn get_master_list_by_state(
        &self,
        state: &str,
    ) -> Result<Vec<MasterListBill>, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&state={state}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getMasterList",
            state = state
        );
        let response = self.client.get(url).send().await.unwrap();
        match crate::handle_legiscan_response(response).await {
            Ok(json) => {
                let masterlist: Vec<MasterListBill> = json["masterlist"]
                    .as_object()
                    .unwrap()
                    .iter()
                    .filter(|(key, _val)| key.parse::<i32>().is_ok())
                    .map(|(_key, value)| {
                        serde_json::from_value(value.to_owned()).unwrap_or_default()
                    })
                    .collect();
                Ok(masterlist)
            }
            Err(e) => Err(e),
        }
    }

    /// Retrieve master bill list for a session
    // 1 hour
    pub async fn get_master_list_by_session(
        &self,
        session_id: i32,
    ) -> Result<Vec<MasterListBill>, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&id={session_id}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getMasterList",
            session_id = session_id
        );
        let response = self.client.get(url).send().await.unwrap();
        match crate::handle_legiscan_response(response).await {
            Ok(json) => {
                let masterlist: Vec<MasterListBill> = json["masterlist"]
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
pub async fn test_get_master_list_by_state() {
    let proxy = LegiscanProxy::new().unwrap();
    let masterlist = proxy.get_master_list_by_state("CO").await.unwrap();
    assert_eq!(masterlist.len() >= 2, true);
}

#[tokio::test]
async fn test_get_master_list_by_session() {
    let proxy = LegiscanProxy::new().unwrap();
    let masterlist = proxy.get_master_list_by_session(1797).await.unwrap();
    assert_eq!(masterlist.len() >= 2, true);
}
