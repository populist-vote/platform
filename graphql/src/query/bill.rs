use async_graphql::{Context, FieldResult, Object};
use db::{Bill, BillSearch};
use sqlx::{Pool, Postgres};

use crate::types::BillResult;

#[derive(Default)]
pub struct BillQuery;

#[Object]
impl BillQuery {
    async fn all_bills(&self, ctx: &Context<'_>) -> FieldResult<Vec<BillResult>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = Bill::index(pool).await?;
        let results = records.into_iter().map(BillResult::from).collect();
        Ok(results)
    }

    async fn bills(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Search by voteStatus, name, or slug")] search: BillSearch,
    ) -> FieldResult<Vec<BillResult>> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = Bill::search(pool, &search).await?;
        let results = records.into_iter().map(BillResult::from).collect();
        Ok(results)
    }
}
