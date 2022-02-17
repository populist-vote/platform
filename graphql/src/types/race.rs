use crate::types::OfficeResult;
use async_graphql::{ComplexObject, Context, FieldResult, SimpleObject, ID};
use db::{
    models::{enums::State, race::Race},
    DateTime, Office,
};
use sqlx::{Pool, Postgres};

#[derive(SimpleObject, Debug, Clone)]
#[graphql(complex)]
pub struct RaceResult {
    id: ID,
    slug: String,
    title: String,
    office_position: String,
    office_id: ID,
    state: Option<State>,
    description: Option<String>,
    ballotpedia_link: Option<String>,
    early_voting_begins_date: Option<chrono::NaiveDate>,
    election_date: Option<chrono::NaiveDate>,
    official_website: Option<String>,
    election_id: Option<ID>,
    created_at: DateTime,
    updated_at: DateTime,
}

#[ComplexObject]
impl RaceResult {
    async fn office(&self, ctx: &Context<'_>) -> FieldResult<OfficeResult> {
        let db_pool = ctx.data_unchecked::<Pool<Postgres>>();
        let record = Office::find_by_id(
            db_pool,
            uuid::Uuid::parse_str(&self.office_id.as_str()).unwrap(),
        )
        .await?;
        Ok(OfficeResult::from(record))
    }
}

impl From<Race> for RaceResult {
    fn from(r: Race) -> Self {
        Self {
            id: ID::from(r.id),
            slug: r.slug,
            title: r.title,
            office_id: ID::from(r.office_id),
            office_position: r.office_position,
            state: r.state,
            description: r.description,
            ballotpedia_link: r.ballotpedia_link,
            early_voting_begins_date: r.early_voting_begins_date,
            election_date: r.election_date,
            official_website: r.official_website,
            election_id: match r.election_id {
                Some(id) => Some(ID::from(id)),
                None => None,
            },
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
