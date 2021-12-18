use crate::VotesmartProxy;
use reqwest::{Error, Response};

pub struct CandidateBio<'a>(pub &'a VotesmartProxy);

impl CandidateBio<'_> {
    /// This method grabs the main bio for each candidate.
    pub async fn get_bio(&self, candidate_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&candidateId={candidate_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "CandidateBio.getBio",
            candidate_id = candidate_id,
        );

        self.0.client.get(url).send().await
    }

    /// This method expands on getBio() by expanding the education, profession, political, orgMembership, and congMembership elements.
    pub async fn get_detailed_bio(&self, candidate_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&candidateId={candidate_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "CandidateBio.getDetailedBio",
            candidate_id = candidate_id,
        );

        self.0.client.get(url).send().await
    }

    /// This method grabs the etended bio for each candidate that has one.
    pub async fn get_addl_bio(&self, candidate_id: i32) -> Result<Response, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&candidateId={candidate_id}&o=JSON",
            base_url = &self.0.base_url,
            key = &self.0.api_key,
            operation = "CandidateBio.getAddlBio",
            candidate_id = candidate_id,
        );

        self.0.client.get(url).send().await
    }
}
