use crate::{Error, LegiscanProxy};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetRollCallResponse {
    pub status: String,
    pub roll_call: RollCall,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
pub struct RollCall {
    pub roll_call_id: i64,
    pub bill_id: i64,
    pub date: String,
    pub desc: String,
    pub yea: i64,
    pub nay: i64,
    pub nv: i64,
    pub absent: i64,
    pub total: i64,
    pub passed: i64,
    pub chamber: String,
    pub chamber_id: i64,
    pub votes: Vec<Vote>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
pub struct Vote {
    pub people_id: i64,
    pub vote_id: i64,
    pub vote_text: String,
}

impl LegiscanProxy {
    /// Retrieve roll call vote information for a given roll_call_id
    // static
    pub async fn get_roll_call(&self, roll_call_id: i32) -> Result<RollCall, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&id={roll_call_id}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getRollCall",
            roll_call_id = roll_call_id
        );
        let response = self.client.get(url).send().await.unwrap();

        match crate::handle_legiscan_response(response).await {
            Ok(json) => {
                let json: GetRollCallResponse = serde_json::from_value(json).unwrap();
                Ok(json.roll_call)
            }
            Err(e) => Err(e),
        }
    }
}

#[tokio::test]
async fn test_get_roll_call() {
    let proxy = LegiscanProxy::new().unwrap();
    let roll_call = proxy.get_roll_call(234223).await.unwrap();
    println!("{}", serde_json::to_string_pretty(&roll_call).unwrap());
    assert_eq!(roll_call.date, "2013-02-20");
}
