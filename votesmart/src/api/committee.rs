use crate::VotesmartProxy;
use reqwest::{Error, Response};

pub struct Committee<'a>(pub &'a VotesmartProxy);

impl Committee<'_> {
    /// Returns the committee types(house, senate, joint, etc)
    pub async fn get_types(&self) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Committee.getTypes",
        );

        self.0.client.get(url).send().await
    }

    /// Returns the list of committees that fit the criteria
    pub async fn get_committees_by_type_state(
        &self,
        committee_type_id: Option<&str>,
        state_id: Option<&str>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&committeeTypeId={committee_type_id}&stateId={state_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Committee.getCommitteesByTypeState",
            committee_type_id = committee_type_id.unwrap_or(""),
            state_id = state_id.unwrap_or("")
        );

        self.0.client.get(url).send().await
    }

    /// Returns detailed committee data.
    pub async fn get_committee(&self, committee_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&committeeId={committee_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Committee.getCommittee",
            committee_id = committee_id
        );

        self.0.client.get(url).send().await
    }

    /// Returns members of the committee
    pub async fn get_committee_members(&self, committee_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&committeeId={committee_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Committee.getCommitteeMembers",
            committee_id = committee_id
        );

        self.0.client.get(url).send().await
    }
}
