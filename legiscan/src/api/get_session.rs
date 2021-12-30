use crate::{Error, LegiscanProxy};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetSessionListResponse {
    pub status: String,
    pub sessions: Vec<Session>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub session_id: i64,
    pub state_id: i64,
    pub year_start: i64,
    pub year_end: i64,
    pub special: i64,
    pub session_name: String,
    pub name: String,
    pub session_hash: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GetSessionPeopleResponse {
    pub status: String,
    #[serde(rename = "sessionpeople")]
    pub session_people: SessionPeople,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionPeople {
    session: Session,
    people: Vec<crate::api::get_person::Person>,
}

impl LegiscanProxy {
    /// Retrieve a list of available sessions for the given state abbreviation
    /// Refresh daily
    /// List of session information including session_id for subsequent getMasterList calls along with session years, special
    /// session indicator and the session_hash which reflects the current dataset version for the session_id for identifying
    /// and tracking when bills change.
    pub async fn get_session_list(&self, state: &str) -> Result<Vec<Session>, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&state={state}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getSessionList",
            state = state
        );
        let response = self.client.get(url).send().await.unwrap();
        match crate::handle_legiscan_response(response).await {
            Ok(json) => {
                let json: GetSessionListResponse = serde_json::from_value(json).unwrap();
                Ok(json.sessions)
            }
            Err(e) => Err(e),
        }
    }

    /// Retrieve list of people active in a specific `session_id`
    // weekly
    pub async fn get_session_people(&self, session_id: i32) -> Result<SessionPeople, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&id={session_id}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getSessionPeople",
            session_id = session_id
        );
        let response = self.client.get(url).send().await.unwrap();

        match crate::handle_legiscan_response(response).await {
            Ok(json) => {
                let json: GetSessionPeopleResponse = serde_json::from_value(json).unwrap();
                Ok(json.session_people)
            }
            Err(e) => Err(e),
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_get_session_list() {
    let proxy = LegiscanProxy::new().unwrap();
    let session_list = proxy.get_session_list("CO").await.unwrap();
    assert_eq!(session_list[0].session_id, 1797);
}

#[tokio::test]
#[ignore]
async fn test_get_session_people() {
    let proxy = LegiscanProxy::new().unwrap();
    let session_people = proxy.get_session_people(1797).await.unwrap();
    assert_eq!(session_people.people.len(), 101);
}
