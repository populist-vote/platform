use crate::VotesmartProxy;
use reqwest::{Error, Response};

pub struct Address<'a>(pub &'a VotesmartProxy);

impl Address<'_> {
    /// This method grabs campaign office(s) and basic candidate information for the specified candidate.
    pub async fn get_campaign(&self, candidate_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&candidateId={candidate_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Address.getCampaign",
            candidate_id = candidate_id
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs the campaign office's Web address(es) and basic candidate information for the specified candidate.
    pub async fn get_campaign_web_address(&self, candidate_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&candidateId={candidate_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Address.getCampaignWebAddress",
            candidate_id = candidate_id
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs campaign office(s) and basic candidate information for the specified election.
    pub async fn get_campaign_by_election(&self, election_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&electionId={election_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Address.getCampaignByElection",
            election_id = election_id
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs office(s) and basic candidate information for the specified candidate.
    pub async fn get_office(&self, candidate_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&candidateId={candidate_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Address.getOffice",
            candidate_id = candidate_id
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs office's Web address(es) and basic candidate information for the specified candidate.
    pub async fn get_office_web_address(&self, candidate_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&candidateId={candidate_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Address.getOfficeWebAddress",
            candidate_id = candidate_id
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs office address and basic candidate information according to the officeId and state.
    pub async fn get_office_by_office_state(
        &self,
        office_id: i32,
        state_id: Option<String>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&officeId={office_id}&stateId={state_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Address.getOfficeByOfficeState",
            office_id = office_id,
            state_id = state_id.unwrap_or("NA".to_string())
        );

        self.0.client.get(url).send().await
    }
}
