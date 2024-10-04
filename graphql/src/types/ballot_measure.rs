use async_graphql::{ComplexObject, Context, Result, SimpleObject, ID};
use db::{
    models::{
        ballot_measure::BallotMeasure,
        enums::{BallotMeasureStatus, State},
    },
    PublicVotes,
};
use uuid::Uuid;

use crate::context::ApiContext;

use super::{ArgumentResult, IssueTagResult};

#[derive(SimpleObject)]
#[graphql(complex)]
pub struct BallotMeasureResult {
    id: ID,
    slug: String,
    title: String,
    status: BallotMeasureStatus,
    election_id: ID,
    state: State,
    ballot_measure_code: String,
    measure_type: String,
    definitions: String,
    description: Option<String>,
    official_summary: Option<String>,
    populist_summary: Option<String>,
    full_text_url: Option<String>,
    yes_votes: Option<i32>,
    no_votes: Option<i32>,
    num_precincts_reporting: Option<i32>,
    total_precincts: Option<i32>,
}

#[ComplexObject]
impl BallotMeasureResult {
    async fn arguments(&self, _ctx: &Context<'_>) -> Result<Vec<ArgumentResult>> {
        //Change to ArgumentResult once implemented
        todo!()
    }

    async fn public_votes(&self, ctx: &Context<'_>) -> Result<PublicVotes> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let results = sqlx::query_as!(
            PublicVotes,
            r#"
                SELECT SUM(CASE WHEN position = 'support' THEN 1 ELSE 0 END) as support,
                       SUM(CASE WHEN position = 'neutral' THEN 1 ELSE 0 END) as neutral,
                       SUM(CASE WHEN position = 'oppose' THEN 1 ELSE 0 END) as oppose
                FROM ballot_measure_public_votes WHERE ballot_measure_id = $1
            "#,
            Uuid::parse_str(&self.id).unwrap(),
        )
        .fetch_one(&db_pool)
        .await?;

        Ok(results)
    }

    async fn issue_tags(&self, ctx: &Context<'_>) -> Result<Vec<IssueTagResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records =
            BallotMeasure::issue_tags(&db_pool, uuid::Uuid::parse_str(&self.id).unwrap()).await?;
        let results = records.into_iter().map(IssueTagResult::from).collect();
        Ok(results)
    }
}

impl From<BallotMeasure> for BallotMeasureResult {
    fn from(b: BallotMeasure) -> Self {
        Self {
            id: ID::from(b.id),
            slug: b.slug,
            title: b.title,
            status: b.status,
            election_id: ID::from(b.election_id),
            state: b.state,
            ballot_measure_code: b.ballot_measure_code,
            measure_type: b.measure_type,
            definitions: b.definitions,
            description: b.description,
            official_summary: b.official_summary,
            populist_summary: b.populist_summary,
            full_text_url: b.full_text_url,
            yes_votes: b.yes_votes,
            no_votes: b.no_votes,
            num_precincts_reporting: b.num_precincts_reporting,
            total_precincts: b.total_precincts,
        }
    }
}
