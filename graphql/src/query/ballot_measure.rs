use async_graphql::{Context, FieldResult, Object};
use db::{BallotMeasure, BallotMeasureSearch};

use crate::{context::ApiContext, types::BallotMeasureResult};

#[derive(Default)]
pub struct BallotMeasureQuery;

#[Object]
impl BallotMeasureQuery {
    async fn all_ballot_measures(
        &self,
        ctx: &Context<'_>,
    ) -> FieldResult<Vec<BallotMeasureResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = BallotMeasure::index(&db_pool).await?;
        let results = records.into_iter().map(BallotMeasureResult::from).collect();
        Ok(results)
    }

    async fn ballot_measures(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search by voteStatus, name, or slug")] search: BallotMeasureSearch,
    ) -> FieldResult<Vec<BallotMeasureResult>> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = BallotMeasure::search(&db_pool, &search).await?;
        let results = records.into_iter().map(BallotMeasureResult::from).collect();
        Ok(results)
    }
}
