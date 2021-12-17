use crate::VotesmartProxy;
use reqwest::{Error, Response};

pub struct Rating<'a>(pub &'a VotesmartProxy);

impl Rating<'_> {
    /// This method dumps categories that contain released ratingss according to state.
    pub async fn get_categories(&self, state_id: Option<String>) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&stateId={state_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Rating.getCategories",
            state_id = state_id.unwrap_or("NA".to_string()),
        );

        self.0.client.get(url).send().await
    }

    /// This method dumps Special Interest Groups according to category and state.
    pub async fn get_sig_list(
        &self,
        category_id: i32,
        state_id: Option<String>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&categoryId={category_id}&stateId={state_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Rating.getSigList",
            category_id = category_id,
            state_id = state_id.unwrap_or("NA".to_string()),
        );

        self.0.client.get(url).send().await
    }

    /// This method dumps detailed information an a Special Interest Group.
    pub async fn get_sig(&self, sig_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&sigId={sig_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Rating.getSig",
            sig_id = sig_id
        );

        self.0.client.get(url).send().await
    }

    /// This method dumps detailed information an a Special Interest Group.
    pub async fn get_sig_ratings(&self, sig_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&sigId={sig_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Rating.getSigRatings",
            sig_id = sig_id
        );

        self.0.client.get(url).send().await
    }

    /// This method dumps a candidate's rating by a Special Interest Group.
    pub async fn get_candidate_rating(
        &self,
        candidate_id: i32,
        sig_id: Option<String>,
    ) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&candidateId={candidate_id}&sigId={sig_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Rating.getCandidateRating",
            candidate_id = candidate_id,
            sig_id = sig_id.unwrap_or("NULL".to_string())
        );

        self.0.client.get(url).send().await
    }

    /// This method dumps all candidate ratings from a scorecard by an Special Interest Group.
    pub async fn get_rating(&self, rating_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&ratingId={rating_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "Rating.getRating",
            rating_id = rating_id
        );

        self.0.client.get(url).send().await
    }
}
