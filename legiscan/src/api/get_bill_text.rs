use crate::{Error, LegiscanProxy};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetBillTextResponse {
    status: String,
    text: BillText,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BillText {
    doc_id: i32,
    bill_id: i32,
    date: String,
    #[serde(rename = "type")]
    type_field: String,
    mime: String,
    doc: String,
}

impl LegiscanProxy {
    // static
    pub async fn get_bill_text(&self, doc_id: i32) -> Result<BillText, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&id={doc_id}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getBillText",
            doc_id = doc_id
        );
        let response = self.client.get(url).send().await.unwrap();

        match crate::handle_legiscan_response(response).await {
            Ok(json) => {
                let json: GetBillTextResponse = serde_json::from_value(json).unwrap();
                Ok(json.text)
            }
            Err(e) => Err(e),
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_get_bill_text() {
    let proxy = LegiscanProxy::new().unwrap();
    let bill_text = proxy.get_bill_text(647508).await.unwrap();
    assert_eq!(bill_text.bill_id, 428345);
    assert_eq!(bill_text.date, "2012-05-23");

    // Doc does not exist
    let result = proxy.get_bill_text(123456789).await;
    assert!(matches!(result, Err(Error::Api(_))));
}
