use crate::{Error, LegiscanProxy};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetAmendmentResponse {
    pub status: String,
    pub amendment: Amendment,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Amendment {
    pub amendment_id: i32,
    pub chamber: String,
    pub chamber_id: i32,
    pub bill_id: i32,
    pub adopted: i32,
    pub date: String,
    pub title: String,
    pub description: String,
    pub mime: String,
    pub mime_id: i32,
    pub doc: String,
}

impl LegiscanProxy {
    /// Retrieve amendment text for a given amendment_id
    // static
    pub async fn get_amendment(&self, amendment_id: i32) -> Result<Amendment, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&id={amendment_id}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getAmendment",
            amendment_id = amendment_id
        );
        let response = self.client.get(url).send().await.unwrap();

        match crate::handle_legiscan_response(response).await {
            Ok(json) => {
                let json: GetAmendmentResponse = serde_json::from_value(json).unwrap();
                Ok(json.amendment)
            }
            Err(e) => Err(e),
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_get_amendment() {
    let proxy = LegiscanProxy::new().unwrap();
    let amendment = proxy.get_amendment(37508).await.unwrap();
    assert_eq!(amendment.title, "Senate Amendment 001");
    assert_eq!(amendment.date, "2016-04-01");

    // Amendment does not exist
    let result = proxy.get_amendment(1231231231).await;
    assert!(matches!(result, Err(Error::Api(_))));
}
