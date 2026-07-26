use axum::{
    extract::{rejection::QueryRejection, Query, State},
    Json,
};
use serde::Serialize;

use super::{
    error::ApiError,
    pagination::{Pagination, PaginationMeta, PaginationQuery},
    RestState,
};

const ENDPOINT: &str = "/api/v1/states";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateResource {
    code: String,
    name: String,
}

impl StateResource {
    pub fn new(code: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            name: name.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct StateCollection {
    data: Vec<StateResource>,
    meta: PaginationMeta,
}

pub async fn list(
    State(state): State<RestState>,
    query: Result<Query<PaginationQuery>, QueryRejection>,
) -> Result<Json<StateCollection>, ApiError> {
    let pagination = Pagination::from_query(query, ENDPOINT)?;
    let (data, meta) = pagination.page(state.states.as_ref());

    Ok(Json(StateCollection { data, meta }))
}
