use crate::VotesmartProxy;
use reqwest::{Error, Response};

pub struct State<'a>(pub &'a VotesmartProxy);

impl State<'_> {
    /// This method grabs a simple state ID and name list for mapping ID to state names.
    pub async fn get_state_ids(&self) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "State.getStateIds",
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs a various data Votesmart keeps on a state
    pub async fn get_state(&self, state_id: String) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&stateId={state_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "State.getStateIds",
            state_id = state_id
        );

        self.0.client.get(url).send().await
    }
}
