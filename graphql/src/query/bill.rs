use async_graphql::{Context, Object};
use db::{Bill, BillFilter, BillSort};

use crate::{context::ApiContext, relay, types::BillResult};

#[derive(Default)]
pub struct BillQuery;

#[Object]
impl BillQuery {
    async fn bills(
        &self,
        ctx: &Context<'_>,
        filter: Option<BillFilter>,
        sort: Option<BillSort>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> relay::ConnectionResult<BillResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Bill::filter(
            &db_pool,
            &filter.unwrap_or_default(),
            &sort.unwrap_or_default(),
        )
        .await?;

        relay::query(
            records.into_iter().map(BillResult::from),
            relay::Params::new(after, before, first, last),
            10,
        )
        .await
    }

    async fn popular_bills(
        &self,
        ctx: &Context<'_>,
        filter: Option<BillFilter>,
        after: Option<String>,
        before: Option<String>,
        first: Option<i32>,
        last: Option<i32>,
    ) -> relay::ConnectionResult<BillResult> {
        let db_pool = ctx.data::<ApiContext>()?.pool.clone();
        let records = Bill::popular(&db_pool, &filter.unwrap_or_default()).await?;

        relay::query(
            records.into_iter().map(BillResult::from),
            relay::Params::new(after, before, first, last),
            10,
        )
        .await
    }

    async fn bill_by_id(&self, ctx: &Context<'_>, id: String) -> Option<BillResult> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        let record = Bill::find_by_id(&db_pool, uuid::Uuid::parse_str(&id).unwrap())
            .await
            .unwrap();
        Some(BillResult::from(record))
    }

    async fn bill_by_slug(&self, ctx: &Context<'_>, slug: String) -> Option<BillResult> {
        let db_pool = ctx.data::<ApiContext>().unwrap().pool.clone();
        let record = Bill::find_by_slug(&db_pool, &slug).await.unwrap();
        Some(BillResult::from(record))
    }
}
