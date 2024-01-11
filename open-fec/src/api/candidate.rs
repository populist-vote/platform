use crate::OpenFecProxy;
use reqwest::{Error, Response};
use serde::{Deserialize, Serialize};

pub struct Candidate<'a>(pub &'a OpenFecProxy);

#[derive(Serialize, Deserialize, Default)]
pub struct CandidateQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub cycle: Option<Vec<u32>>,
    pub election_year: Option<u32>,
    pub office: Option<String>,
    pub state: Option<String>,
    pub party: Option<Vec<String>>,
    pub year: Option<String>,
    pub district: Option<Vec<String>>,
    pub candidate_status: Option<Vec<String>>,
    pub incumbent_challenge: Option<Vec<String>>,
    pub federal_funds_flag: Option<bool>,
    pub has_raised_funds: Option<bool>,
    pub name: Option<Vec<String>>,
    pub sort: Option<String>,
    pub sort_hide_null: Option<bool>,
    pub sort_null_only: Option<bool>,
    pub sort_nulls_last: Option<bool>,
    pub api_key: Option<String>,
}

impl Candidate<'_> {
    pub async fn get_candidate(
        &self,
        candidate_id: &str,
        query: CandidateQuery,
    ) -> Result<Response, Error> {
        let path = format!("/v1/candidate/{}", candidate_id);
        self.0.get(&path, query).await
    }
}
