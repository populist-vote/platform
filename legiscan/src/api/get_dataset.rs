use crate::{Error, LegiscanProxy};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetDatasetResponse {
    pub status: String,
    pub dataset: Dataset,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dataset {
    pub state_id: i64,
    pub session_id: i64,
    pub session_name: String,
    pub dataset_hash: String,
    pub dataset_date: String,
    pub dataset_size: i64,
    pub mime_type: String,
    pub zip: String,
}

impl LegiscanProxy {
    /// Retrieve an individual dataset for a specific `session_id`
    // weekly
    pub async fn get_dataset(&self, session_id: i32, access_key: &str) -> Result<Dataset, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&id={session_id}&access_key={access_key}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getDataset",
            session_id = session_id,
            access_key = access_key
        );
        println!("{}", url);

        let response = self.client.get(url).send().await.unwrap();

        match crate::handle_legiscan_response(response).await {
            Ok(json) => {
                let json: GetDatasetResponse = serde_json::from_value(json).unwrap();
                Ok(json.dataset)
            }
            Err(e) => Err(e),
        }
    }
}

#[tokio::test]
async fn test_get_dataset() {
    let proxy = LegiscanProxy::new().unwrap();
    let dataset = proxy
        .get_dataset(1789, "55jFZUhExATO7PdWI5vJJS")
        .await
        .unwrap();
    assert_eq!(dataset.state_id, 6);
    assert_eq!(dataset.session_name, "2020 First Special Session");

    // Dataset doesnt exist
    let result = proxy.get_dataset(1234, "some-bad-key").await;
    assert!(matches!(result, Err(Error::Api(_))));
}
