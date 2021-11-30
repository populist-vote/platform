use crate::{Error, GetCandidateBioResponse};

const VOTESMART_BASE_URL: &str = "http://api.votesmart.org/";

/// Stuct used to make calls to the Votesmart API
pub struct VotesmartProxy {
    client: reqwest::Client,
    pub base_url: reqwest::Url,
    api_key: String,
}

impl VotesmartProxy {
    pub fn new() -> Result<Self, Error> {
        dotenv::dotenv().ok();
        let api_key = std::env::var("VOTESMART_API_KEY")?;
        let client = reqwest::Client::new();

        Ok(VotesmartProxy {
            client,
            base_url: reqwest::Url::parse(VOTESMART_BASE_URL).unwrap(),
            api_key,
        })
    }

    pub async fn get_candidate_bio(
        &self,
        candidate_id: i32,
    ) -> Result<GetCandidateBioResponse, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&candidateId={candidate_id}&o=JSON",
            base_url = self.base_url,
            key = self.api_key,
            operation = "CandidateBio.getDetailedBio",
            candidate_id = candidate_id,
        );

        let response = self.client.get(url).send().await.unwrap();

        if response.status().is_success() {
            let json: serde_json::Value = response.json().await?;
            let bio = &json["bio"];
            match bio {
                serde_json::Value::Null => {
                    println!(
                        "Candidate with id {} does not exist in the Votesmart API",
                        candidate_id
                    );
                    std::process::exit(0)
                }
                _ => Ok(serde_json::from_value(bio.to_owned()).unwrap()),
            }
        } else {
            Err(Error::ApiError)
        }
    }
}
