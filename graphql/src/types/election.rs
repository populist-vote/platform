use async_graphql::{ComplexObject, ID, SimpleObject};
use db::{Election};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct ElectionResult {
    id: ID,
    slug: String,
    title: String,
    description: Option<String>,
    election_date: chrono::NaiveDate
}

#[ComplexObject]
impl ElectionResult { }

impl From<Election> for ElectionResult {
    fn from(e: Election) -> Self {
        Self {
            id: ID::from(e.id),
            slug: e.slug,
            title: e.title,
            description: e.description,
            election_date: e.election_date,
        }
    }
}
