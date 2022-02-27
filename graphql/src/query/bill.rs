use async_graphql::{Context, Object};
use db::{Bill, BillSearch};

use crate::{context::ApiContext, relay, types::BillResult};

#[derive(Default)]
pub struct BillQuery;

#[Object]
impl BillQuery {
    async fn bills(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search by voteStatus, title, or slug", default)] search: BillSearch,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> relay::ConnectionResult<BillResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Bill::search(&db_pool, &search).await?;
        let results = records.into_iter().map(BillResult::from);

        relay::query(
            results.into_iter(),
            relay::Params::new(after, before, first, last),
            10,
        )
        .await
    }
}
