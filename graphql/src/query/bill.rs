use async_graphql::{Context, FieldResult, Object};
use db::{Bill, BillSearch};
use sqlx::{Pool, Postgres};

use crate::{connection, types::BillResult};

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
    ) -> connection::ConnectionResult<BillResult> {
        let pool = ctx.data_unchecked::<Pool<Postgres>>();
        let records = Bill::search(pool, &search).await?;
        let results = records.into_iter().map(BillResult::from);

        connection::query(
            results.into_iter(),
            connection::Params::new(after, before, first, last),
            10,
        )
        .await
    }
}
