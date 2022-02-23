use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VsRating {
    pub categories: Value,
    pub rating: Value,
    pub rating_id: Option<Value>,
    pub rating_name: String,
    pub rating_text: String,
    pub sig_id: Value,
    pub timespan: Value,
}
#[derive(SimpleObject, Debug, Clone, Serialize, Deserialize)]
pub struct VsCategoryItem {
    pub category_id: i32,
    pub name: Value,
}
