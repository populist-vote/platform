use crate::{Error, LegiscanProxy, RollCall};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct GetBillResponse {
    status: String,
    bill: Bill,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    feature = "async-graphql",
    derive(async_graphql::SimpleObject),
    graphql(complex)
)]
pub struct Bill {
    pub bill_id: i32,
    pub change_hash: String,
    pub session_id: i32,
    pub session: Session,
    pub url: String,
    pub state_link: String,
    pub completed: i32,
    pub status: i32,
    pub status_date: Option<String>,
    pub progress: Vec<Progress>,
    pub state: String,
    pub state_id: i32,
    pub bill_number: String,
    pub bill_type: String,
    pub bill_type_id: String,
    pub body: String,
    pub body_id: i32,
    pub current_body: String,
    pub current_body_id: i32,
    pub title: String,
    pub committee: serde_json::Value, // sometimes a Commitee, sometimes an empty array :(
    pub referrals: Option<Vec<Referral>>,
    pub pending_committee_id: i32,
    pub history: Vec<History>,
    pub sponsors: Vec<Sponsor>,
    pub sasts: Vec<Sast>,
    pub subjects: Vec<Subject>,
    pub texts: Vec<Text>,
    pub votes: Vec<Vote>,
    pub amendments: Vec<Amendment>,
    pub supplements: Vec<Supplement>,
    pub calendar: Vec<Calendar>,
}

#[cfg_attr(feature = "async-graphql", async_graphql::ComplexObject)]
impl Bill {
    #[cfg(feature = "async-graphql")]
    async fn status_type(&self) -> async_graphql::FieldResult<String> {
        Ok(BillStatus::try_from(self.status).unwrap().to_string())
    }
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    num_enum::TryFromPrimitive,
    strum_macros::Display,
)]
#[repr(i32)]
pub enum BillStatus {
    // This is rare but does show up from time to time
    NotIntroduced = 0,
    Introduced = 1,
    Engrossed = 2,
    Enrolled = 3,
    Passed = 4,
    Vetoed = 5,
    /// Limited support based on state
    Failed = 6,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
pub struct Session {
    pub session_id: i32,
    pub session_name: String,
    pub session_title: String,
    pub year_start: i32,
    pub year_end: i32,
    pub special: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
pub struct Progress {
    pub date: String,
    pub event: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
pub struct Committee {
    pub committee_id: i32,
    pub chamber: String,
    pub chamber_id: i32,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
pub struct Referral {
    pub date: String,
    pub committee_id: i32,
    pub chamber: String,
    pub chamber_id: i32,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
pub struct History {
    pub date: String,
    pub action: String,
    pub chamber: String,
    pub chamber_id: i32,
    pub importance: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
pub struct Sponsor {
    pub people_id: i32,
    pub person_hash: String,
    pub party_id: serde_json::Value, // Sometimes a string, sometimes an i32
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
    pub sponsor_type_id: i32,
    pub sponsor_order: i32,
    pub committee_sponsor: i32,
    pub committee_id: serde_json::Value, // Sometimes a string, sometimes an i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
pub struct Sast {
    pub type_id: i32,
    #[serde(rename = "type")]
    pub type_field: String,
    pub sast_bill_number: String,
    pub sast_bill_id: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
pub struct Subject {
    pub subject_id: i32,
    pub subject_name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
pub struct Text {
    pub doc_id: i32,
    pub date: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub type_id: i32,
    pub mime: String,
    pub mime_id: i32,
    pub url: String,
    pub state_link: String,
    pub text_size: i32,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
pub struct Vote {
    pub roll_call_id: i32,
    /// This field is not returned from get_bill, but can be populated with a subsequent call to `get_roll_call`
    pub roll_call_data: Option<RollCall>,
    pub date: String,
    pub desc: String,
    pub yea: i32,
    pub nay: i32,
    pub nv: i32,
    pub absent: i32,
    pub total: i32,
    pub passed: i32,
    pub chamber: String,
    pub chamber_id: i32,
    pub url: String,
    pub state_link: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
pub struct Amendment {
    pub amendment_id: i32,
    pub adopted: i32,
    pub chamber: String,
    pub chamber_id: i32,
    pub date: String,
    pub title: String,
    pub description: String,
    pub mime: String,
    pub mime_id: i32,
    pub url: String,
    pub state_link: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
pub struct Supplement {
    pub supplement_id: i32,
    pub date: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub type_id: i32,
    pub title: String,
    pub description: String,
    pub mime: String,
    pub mime_id: i32,
    pub url: String,
    pub state_link: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "async-graphql", derive(async_graphql::SimpleObject))]
pub struct Calendar {
    pub type_id: i32,
    #[serde(rename = "type")]
    pub type_field: String,
    pub date: String,
    pub time: String,
    pub location: String,
    pub description: String,
}

impl LegiscanProxy {
    // 3 hours
    pub async fn get_bill(&self, bill_id: i32) -> Result<Bill, Error> {
        let url = format!(
            "{base_url}?key={key}&op={operation}&id={bill_id}",
            base_url = self.base_url,
            key = self.api_key,
            operation = "getBill",
            bill_id = bill_id
        );

        let response = self.client.get(url).send().await.unwrap();

        match crate::handle_legiscan_response(response).await {
            Ok(json) => {
                let json: GetBillResponse = serde_json::from_value(json.clone())
                    .unwrap_or_else(|_| panic!("Heres the crappy JSON: {}", &json));
                Ok(json.bill)
            }
            Err(e) => Err(e),
        }
    }
}

#[tokio::test]
async fn test_get_bill() {
    let proxy = LegiscanProxy::new().unwrap();
    let bill = proxy.get_bill(428345).await.unwrap();
    assert_eq!(bill.title, "Joint Resolution disapproving permanent rules of the Oklahoma Department of Agriculture, Food, and Forestry; distribution.");

    // Lets test a couple more bills, they don't always have the same shape
    let bill = proxy.get_bill(1369163).await.unwrap();
    assert_eq!(bill.title, "Social Equity Licensees In Regulated Marijuana");

    let bill = proxy.get_bill(1268877).await.unwrap();
    assert_eq!(bill.title, "Enacts the farm laborers fair labor practices act: grants collective bargaining rights to farm laborers; requires employers of farm laborers to allow at least 24 consecutive hours of rest each week; provides for an 8 hour work day for farm laborers; requires overtime rate at one and one-half times normal rate; makes provisions of unemployment insurance law applicable to farm laborers; provides sanitary code shall apply to all farm and food processing labor camps intended to house migrant workers, regardless of the number of occupants; provides for eligibility of farm laborers for workers' compensation benefits; requires employers of farm laborers to provide such farm laborers with claim forms for workers' compensation claims under certain conditions; requires reporting of injuries to employers of farm laborers.");

    // Bill does not exist
    let result = proxy.get_bill(123456789).await;
    assert!(matches!(result, Err(Error::Api(_))));
}
