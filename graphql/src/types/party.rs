use async_graphql::{SimpleObject, ID};

#[derive(Clone, SimpleObject, Debug)]
pub struct PoliticalParty {
    pub id: ID,
    pub slug: String,
    pub fec_code: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub notes: Option<String>,
}
