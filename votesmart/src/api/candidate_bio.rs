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

#[cfg(test)]
mod tests {
    use crate::VotesmartProxy;

    #[tokio::test]
    async fn test_get_bio() {
        let proxy = VotesmartProxy::new().unwrap();
        let response = proxy.candidate_bio().get_bio(110942).await.unwrap();
        assert_eq!(response.status().is_success(), true);
        let json: serde_json::Value = response.json().await.unwrap();
        // println!("{}", serde_json::to_string_pretty(&json).unwrap());
        assert_eq!(json["bio"]["candidate"]["firstName"], "Michael");
    }

    #[tokio::test]
    async fn test_get_detailed_bio() {
        let proxy = VotesmartProxy::new().unwrap();
        let response = proxy.candidate_bio().get_detailed_bio(53279).await.unwrap();
        assert_eq!(response.status().is_success(), true);
        let json: serde_json::Value = response.json().await.unwrap();
        // println!("{}", serde_json::to_string_pretty(&json).unwrap());
        assert_eq!(
            json["bio"]["candidate"]["political"]["experience"][0]["title"],
            "President"
        );
    }

    #[tokio::test]
    async fn test_get_addl_bio() {
        let proxy = VotesmartProxy::new().unwrap();
        let response = proxy.candidate_bio().get_detailed_bio(53279).await.unwrap();
        assert_eq!(response.status().is_success(), true);
        let json: serde_json::Value = response.json().await.unwrap();
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
        assert_eq!(
            json["bio"]["candidate"]["political"]["experience"][0]["title"],
            "President"
        );
    }
}
