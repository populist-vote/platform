use crate::VotesmartProxy;
use reqwest::{Error, Response};

pub struct Leadership<'a>(pub &'a VotesmartProxy);

impl Leadership<'_> {
    /// Gets leadership positions by state and office
    pub async fn get_positions(&self) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Leadership.getPositions",
        );

        self.0.client.get(url).send().await
    }

    /// Gets officials that hold the leadership role in certain states.
    pub async fn get_officials(
        &self,
        leadership_id: i32,
        state_id: Option<&str>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&leadershipId={leadership_id}&stateId={state_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Leadership.getOfficials",
            leadership_id = leadership_id,
            state_id = state_id.unwrap_or("")
        );

        self.0.client.get(url).send().await
    }
}
