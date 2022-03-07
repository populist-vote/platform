use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{
    models::enums::{PoliticalParty, RaceType, State},
    Election, Race,
};

use crate::context::ApiContext;

use super::RaceResult;

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct ElectionResult {
    id: ID,
    slug: String,
    title: String,
    description: Option<String>,
    election_date: chrono::NaiveDate,
}

#[ComplexObject]
impl ElectionResult {
    async fn races(&self, ctx: &Context<'_>) -> Result<Vec<RaceResult>> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        let records = sqlx::query_as!(
            Race,
            r#"
            SELECT id, slug, title, office_position, office_id, race_type AS "race_type:RaceType", party AS "party:PoliticalParty", state AS "state:State", description, ballotpedia_link, early_voting_begins_date, election_date, official_website, election_id, created_at, updated_at FROM race 
            WHERE election_id = $1
        "#,
            uuid::Uuid::parse_str(&self.id).unwrap()
        )
        .fetch_all(&db_pool)
        .await.unwrap();

        let results = records.into_iter().map(RaceResult::from).collect();
        Ok(results)
    }
}

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
