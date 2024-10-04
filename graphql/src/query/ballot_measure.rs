use async_graphql::{Context, Object};
use db::{BallotMeasure, BallotMeasureFilter, BallotMeasureSort};

use crate::{context::ApiContext, relay, types::BallotMeasureResult};

#[derive(Default)]
pub struct BallotMeasureQuery;

#[Object]
impl BallotMeasureQuery {
    async fn ballot_measures(
        &self,
        ctx: &Context<'_>,
        filter: Option<BallotMeasureFilter>,
        sort: Option<BallotMeasureSort>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> relay::ConnectionResult<BallotMeasureResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = BallotMeasure::filter(
            &db_pool,
            &filter.unwrap_or_default(),
            &sort.unwrap_or_default(),
        )
        .await?;

        relay::query(
            records.into_iter().map(BallotMeasureResult::from),
            relay::Params::new(after, before, first, last),
            10,
        )
        .await
    }
}
