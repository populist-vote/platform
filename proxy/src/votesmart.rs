use serde::{Deserialize, Serialize};

use crate::Error;

const VOTESMART_BASE_URL: &str = "http://api.votesmart.org/";

/// Stuct used to make calls to the Votesmart API
pub struct VotesmartProxy {
    client: reqwest::Client,
    pub base_url: reqwest::Url,
    api_key: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCandidateBioResponse {
    pub bio: Bio,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bio {
    pub general_info: GeneralInfo,
    pub candidate: Candidate,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralInfo {
    pub title: String,
    pub link_back: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub candidate_id: String,
    pub crp_id: String,
    pub photo: String,
    pub first_name: String,
    pub nick_name: String,
    pub middle_name: String,
    pub preferred_name: String,
    pub last_name: String,
    pub suffix: String,
    pub birth_date: String,
    pub birth_place: String,
    pub pronunciation: String,
    pub gender: String,
    pub family: String,
    pub home_city: String,
    pub home_state: String,
    pub education: Education,
    pub profession: Profession,
    pub political: Political,
    pub cong_membership: String,
    pub org_membership: OrgMembership,
    pub religion: String,
    pub special_msg: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Education {
    pub institution: Vec<Institution>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Institution {
    pub degree: String,
    pub field: String,
    pub school: String,
    pub span: String,
    pub gpa: String,
    pub full_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profession {
    pub experience: Vec<Experience>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Experience {
    pub title: String,
    pub organization: String,
    pub span: String,
    pub special: String,
    pub district: String,
    pub full_text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Political {
    pub experience: Vec<Experience>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgMembership {
    pub experience: Vec<Experience>,
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
        candidate_id: String,
    ) -> Result<serde_json::Value, Error> {
        let url = format!(
            "{base_url}{operation}?key={key}&candidateId={candidate_id}&o=JSON",
            base_url = self.base_url,
            key = self.api_key,
            operation = "CandidateBio.getDetailedBio",
            candidate_id = candidate_id,
        );

        let response = self.client.get(url).send().await.unwrap();
        let json: serde_json::Value = response.json().await?;
        // let bio: GetCandidateBioResponse = serde_json::from_value(json).unwrap();
        Ok(json)
    }
}
