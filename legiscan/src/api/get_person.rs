use crate::{Error, LegiscanProxy};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetPersonResponse {
    pub status: String,
    pub person: Person,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Person {
    pub people_id: i32,
    pub person_hash: String,
    pub state_id: i32,
    pub party_id: String,
    pub party: String,
    pub role_id: i32,
    pub role: String,
    pub name: String,
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
    pub suffix: String,
    pub nickname: String,
    pub district: String,
    pub ftm_eid: i32,
    pub votesmart_id: i32,
    pub opensecrets_id: String,
    pub ballotpedia: String,
    pub committee_sponsor: i32,
    pub committee_id: i32,
}

impl LegiscanProxy {
    /// Retrieve basic information for a given people_id
    // weekly
    pub async fn get_person(&self, person_id: i32) -> Result<Person, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&id={person_id}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getPerson",
            person_id = person_id
        );
        let response = self.client.get(url).send().await.unwrap();
        match crate::handle_legiscan_response(response).await {
            Ok(json) => {
                let json: GetPersonResponse = serde_json::from_value(json).unwrap();
                Ok(json.person)
            }
            Err(e) => Err(e),
        }
    }
}

#[tokio::test]
async fn test_get_person() {
    let proxy = LegiscanProxy::new().unwrap();
    let person = proxy.get_person(16789).await.unwrap();
    assert_eq!(person.first_name, "Christine");
    assert_eq!(person.votesmart_id, 154843);
}
