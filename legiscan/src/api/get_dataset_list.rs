use crate::{Error, LegiscanProxy};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetDatasetListResponse {
    pub status: String,
    #[serde(rename = "datasetlist")]
    pub dataset_list: Vec<Dataset>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dataset {
    pub state_id: i32,
    pub session_id: i32,
    pub special: i32,
    pub year_start: i32,
    pub year_end: i32,
    pub session_name: String,
    pub session_title: String,
    pub dataset_hash: String,
    pub dataset_date: String,
    pub dataset_size: i32,
    pub access_key: String,
}

impl LegiscanProxy {
    /// Retrieve list of available dataset snapshots
    // weekly
    pub async fn get_dataset_list(
        &self,
        state: Option<&str>,
        year: Option<&str>,
    ) -> Result<Vec<Dataset>, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&state={state}&year={year}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getDatasetList",
            state = state.unwrap_or(""),
            year = year.unwrap_or("")
        );
        let response = self.client.get(url).send().await.unwrap();

        match crate::handle_legiscan_response(response).await {
            Ok(json) => {
                let json: GetDatasetListResponse = serde_json::from_value(json).unwrap();
                Ok(json.dataset_list)
            }
            Err(e) => Err(e),
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_get_dataset_list() {
    let proxy = LegiscanProxy::new().unwrap();
    let dataset_list = proxy
        .get_dataset_list(Some("CO"), Some("2020"))
        .await
        .unwrap();
    assert_eq!(dataset_list.len(), 2);
    assert_eq!(dataset_list[0].session_name, "2020 First Special Session");
}
