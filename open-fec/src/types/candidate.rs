use crate::OpenFecProxy;
use serde::{Deserialize, Serialize};

pub struct Candidate<'a>(pub &'a OpenFecProxy);

#[derive(Serialize, Deserialize, Default)]
pub struct CandidateQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub cycle: Option<Vec<u64>>,
    pub election_year: Option<u64>,
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

#[derive(Serialize, Deserialize, Default)]
pub struct CandidatesQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub q: Option<String>,
    pub candidate_id: Option<Vec<String>>,
    pub min_first_file_date: Option<String>,
    pub max_first_file_date: Option<String>,
    pub is_active_candidate: Option<bool>,
    pub cycle: Option<Vec<u64>>,
    pub election_year: Option<u64>,
    pub office_sought: Option<String>,
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
