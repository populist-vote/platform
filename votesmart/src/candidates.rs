use crate::VotesmartProxy;
use chrono::Datelike;
use reqwest::{Error, Response};

pub struct Candidates<'a>(pub &'a VotesmartProxy);

impl Candidates<'_> {
    /// This method grabs a list of candidates according to office and state representation.
    pub async fn get_by_office_state(
        &self,
        office_id: i32,
        state_id: Option<String>,
        election_year: Option<i32>,
        stage_id: Option<String>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&officeId={office_id}&stateId={state_id}&electionYear={election_year}&stageId={stage_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Candidates.getByOfficeState",
            office_id = office_id,
            state_id = state_id.unwrap_or("NA".to_string()),
            election_year = election_year.unwrap_or(chrono::Utc::now().year()),
            stage_id = stage_id.unwrap_or("NULL".to_string())
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs a list of candidates according to office type and state representation.
    pub async fn get_by_office_state_type(
        &self,
        office_type_id: i32,
        state_id: Option<String>,
        election_year: Option<i32>,
        stage_id: Option<String>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&officeTypeId={office_type_id}&stateId={state_id}&electionYear={election_year}&stageId={stage_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Candidates.getByOfficeStateType",
            office_type_id = office_type_id,
            state_id = state_id.unwrap_or("NA".to_string()),
            election_year = election_year.unwrap_or(chrono::Utc::now().year()),
            stage_id = stage_id.unwrap_or("NULL".to_string())
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs a list of candidates according to a lastname match.
    pub async fn get_by_last_name(
        &self,
        last_name: String,
        election_year: Option<i32>,
        stage_id: Option<String>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&lastName={last_name}&electionYear={election_year}&stageId={stage_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Candidates.getByLastName",
            last_name = last_name,
            election_year = election_year.unwrap_or(chrono::Utc::now().year()),
            stage_id = stage_id.unwrap_or("NULL".to_string())
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs a list of candidates according to a fuzzy lastname match.
    pub async fn get_by_levenshtein(
        &self,
        last_name: String,
        election_year: Option<i32>,
        stage_id: Option<String>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&lastName={last_name}&electionYear={election_year}&stageId={stage_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Candidates.getByLevenshtein",
            last_name = last_name,
            election_year = election_year.unwrap_or(chrono::Utc::now().year()),
            stage_id = stage_id.unwrap_or("NULL".to_string())
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs a list of candidates according to the election they are running in.
    pub async fn get_by_election(
        &self,
        election_id: i32,
        stage_id: Option<String>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&electionId={election_id}&stageId={stage_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Candidates.getByElection",
            election_id = election_id,
            stage_id = stage_id.unwrap_or("NULL".to_string())
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs a list of candidates according to the district they represent.
    pub async fn get_by_district(
        &self,
        district_id: i32,
        election_year: Option<i32>,
        stage_id: Option<String>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&districtId={district_id}&electionYear={election_year}&stageId={stage_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Candidates.getByDistrict",
            district_id = district_id,
            election_year = election_year.unwrap_or(chrono::Utc::now().year()),
            stage_id = stage_id.unwrap_or("NULL".to_string())
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs a list of candidates according to the zip code they represent.
    pub async fn get_by_zip(
        &self,
        zip5: i32,
        zip4: Option<String>,
        stage_id: Option<String>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&zip5={zip5}&zip4={zip4}&stageId={stage_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Candidates.getByZip",
            zip5 = zip5,
            zip4 = zip4.unwrap_or("NULL".to_string()),
            stage_id = stage_id.unwrap_or("NULL".to_string())
        );

        self.0.client.get(url).send().await
    }
}
