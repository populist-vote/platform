use async_graphql::{Context, FieldResult, Object};
use db::{BallotMeasure, BallotMeasureSearch};
use sqlx::{Pool, Postgres};

use crate::types::BallotMeasureResult;

#[derive(Default)]
pub struct BallotMeasureQuery;

#[Object]
impl BallotMeasureQuery {
    async fn all_ballot_measures(
        &self,
        ctx: &Context<'_>,
    ) -> FieldResult<Vec<BallotMeasureResult>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = BallotMeasure::index(pool).await?;
        let results = records.into_iter().map(BallotMeasureResult::from).collect();
        Ok(results)
    }

    async fn ballot_measures(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search by voteStatus, name, or slug")] search: BallotMeasureSearch,
    ) -> FieldResult<Vec<BallotMeasureResult>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = BallotMeasure::search(pool, &search).await?;
        let results = records.into_iter().map(BallotMeasureResult::from).collect();
        Ok(results)
    }
}
