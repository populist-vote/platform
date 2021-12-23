use crate::VotesmartProxy;
use reqwest::{Error, Response};

pub struct Officials<'a>(pub &'a VotesmartProxy);

impl Officials<'_> {
    /// This method grabs a list of officials according to state representation
    pub async fn get_statewide(&self, state_id: String) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&stateId={state_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Officials.getStatewide",
            state_id = state_id,
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs a list of officials according to office and state representation.
    pub async fn get_by_office_state(
        &self,
        office_id: i32,
        state_id: Option<&str>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&officeId={office_id}&stateId={state_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Officials.getByOfficeState",
            office_id = office_id,
            state_id = state_id.unwrap_or(""),
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs a list of officials according to office type and state representation.
    pub async fn get_by_office_type_state(
        &self,
        office_type_id: i32,
        state_id: Option<&str>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&officeTypeId={office_type_id}&stateId={state_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Officials.getByOfficeTypeState",
            office_type_id = office_type_id,
            state_id = state_id.unwrap_or(""),
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs a list of officials according to a lastName match.
    pub async fn get_by_last_name(&self, last_name: String) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&lastName={last_name}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Officials.getByLastname",
            last_name = last_name
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs a list of officials according to a fuzzy lastName match.
    pub async fn get_by_levenshtein(&self, last_name: String) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&lastName={last_name}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Officials.getByLevenshtein",
            last_name = last_name
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs a list of officials according to the district they are running for.
    pub async fn get_by_district(&self, district_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&districtId={district_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Officials.getByDistrict",
            district_id = district_id
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs a list of officials according to the zip code they represent.
    pub async fn get_by_zip(&self, zip5: i32, zip4: Option<&str>) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&zip5={zip5}&zip4={zip4}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Officials.getByZip",
            zip5 = zip5,
            zip4 = zip4.unwrap_or("")
        );

        self.0.client.get(url).send().await
    }
}
