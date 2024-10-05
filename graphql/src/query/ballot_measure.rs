use async_graphql::{Context, Object, ID};
use db::{BallotMeasure, BallotMeasureFilter, BallotMeasureSort};

use crate::{context::ApiContext, relay, types::BallotMeasureResult};

#[derive(Default)]
pub struct BallotMeasureQuery;

#[Object]
impl BallotMeasureQuery {
    async fn ballot_measure_by_id(&self, ctx: &Context<'_>, id: ID) -> Option<BallotMeasureResult> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        let record = BallotMeasure::find_by_id(&db_pool, uuid::Uuid::parse_str(&id).unwrap())
            .await
            .unwrap();
        Some(BallotMeasureResult::from(record))
    }

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
