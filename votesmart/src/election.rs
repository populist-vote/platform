use crate::VotesmartProxy;
use chrono::Datelike;
use reqwest::{Error, Response};

pub struct Election<'a>(pub &'a VotesmartProxy);

impl Election<'_> {
    /// This method grabs district basic election data according to electionId.
    pub async fn get_election(&self, election_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&electionId={election_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Election.getElection",
            election_id = election_id
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs district basic election data according to year and stateid.
    pub async fn get_election_by_year_state(
        &self,
        year: i32,
        state_id: Option<&str>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&year={year}&stateId={state_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Election.getElectionByYearState",
            year = year,
            state_id = state_id.unwrap_or("")
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs district basic election data according to zip code.
    pub async fn get_election_by_zip(
        &self,
        zip5: i32,
        zip4: Option<&str>,
        year: Option<i32>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&zip5={zip5}&zip4={zip4}&year={year}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Election.getElectionByZip",
            zip5 = zip5,
            zip4 = zip4.unwrap_or(""),
            year = year.unwrap_or_else(|| chrono::Utc::now().year()),
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs district basic election data according to electionId and stageId. Per state lists of a Presidential election are available by specifying the stateId.
    pub async fn get_stage_candidates(
        &self,
        election_id: i32,
        stage_id: i32,
        major: String,
        party: Option<&str>,
        district_id: Option<&str>,
        state_id: Option<&str>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&electionId={election_id}&stageId={stage_id}&major={major}&party={party}&districtId={district_id}&stateId={state_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Election.getStageCandidates",
            election_id = election_id,
            stage_id = stage_id,
            major = major,
            party = party.unwrap_or(""),
            district_id = district_id.unwrap_or(""),
            state_id = state_id.unwrap_or(""),
        );

        self.0.client.get(url).send().await
    }
}
