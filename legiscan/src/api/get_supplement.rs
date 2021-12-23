use crate::{Error, LegiscanProxy};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetSupplementResponse {
    status: String,
    supplement: Supplement,
}

#[derive(Serialize, Deserialize)]
pub struct Supplement {
    supplement_id: i32,
    bill_id: i32,
    date: String,
    type_id: i32,
    #[serde(rename = "type")]
    type_field: String,
    title: String,
    description: String,
    mime: String,
    mime_id: i32,
    doc: String,
}

impl LegiscanProxy {
    /// Retrieve supplemental document for a given supplement_id
    // static
    pub async fn get_supplement(&self, supplement_id: i32) -> Result<Supplement, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&id={supplement_id}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getSupplement",
            supplement_id = supplement_id
        );
        let response = self.client.get(url).send().await.unwrap();
        match crate::handle_legiscan_response(response).await {
            Ok(json) => {
                let json: GetSupplementResponse = serde_json::from_value(json).unwrap();
                Ok(json.supplement)
            }
            Err(e) => Err(e),
        }
    }
}

#[tokio::test]
async fn test_get_supplement() {
    let proxy = LegiscanProxy::new().unwrap();
    let supplement = proxy.get_supplement(47508).await.unwrap();
    assert_eq!(supplement.type_field, "Fiscal Note/Analysis");
}
