use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct GetCandidateVotingRecordResponse {
    pub bills: Bills,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct Bills {
    pub general_info: GeneralInfo,
    pub bill: Vec<Bill>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct GeneralInfo {
    pub title: String,
    pub link_back: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct Bill {
    pub bill_id: String,
    pub bill_number: String,
    pub title: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub categories: Categories,
    pub office_id: String,
    pub office: String,
    pub action_id: String,
    pub stage: String,
    pub vote: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, SimpleObject)]
#[serde(rename_all = "camelCase")]
pub struct Categories {
    pub category: Value,
}
