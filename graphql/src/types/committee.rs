use async_graphql::{SimpleObject, ID};
use db::models::{committee::Committee, enums::State};

#[derive(SimpleObject)]
pub struct CommitteeResult {
    id: ID,
    name: String,
    description: String,
    state: Option<State>,
    chair_id: Option<ID>,
    legiscan_committee_id: Option<i32>,
}

impl From<Committee> for CommitteeResult {
    fn from(committee: Committee) -> Self {
        Self {
            id: ID(committee.id.to_string()),
            name: committee.name,
            description: committee.description,
            state: committee.state,
            chair_id: committee.chair_id.map(|id| ID(id.to_string())),
            legiscan_committee_id: committee.legiscan_committee_id,
        }
    }
}
